#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use at32f4xx_pac as pac;

mod he_logic;
mod hw;

use he_logic::HallKey;

const NUM_KEYS: usize = 64;

// DMA buffer for ADC results. Must be static for safe DMA access.
static mut ADC_BUFFER: [u16; NUM_KEYS] = [0; NUM_KEYS];

const RGB_COUNT: usize = NUM_KEYS;
const RGB_BUF_LEN: usize = (RGB_COUNT * 24) + 80; // 24 bits + 100us reset period
static mut RGB_BUFFER: [u16; RGB_BUF_LEN] = [0; RGB_BUF_LEN];

/// Set an LED color in the DMA buffer (GRB order for SK6812-E)
fn set_led(idx: usize, r: u8, g: u8, b: u8) {
    if idx >= RGB_COUNT { return; }
    let start = idx * 24;
    unsafe {
        for bit in 0..8 {
            // SK6812-E specific timings (216MHz clock)
            RGB_BUFFER[start + (7 - bit)] = if (g >> bit) & 1 == 1 { 130 } else { 65 };
            RGB_BUFFER[start + 8 + (7 - bit)] = if (r >> bit) & 1 == 1 { 130 } else { 65 };
            RGB_BUFFER[start + 16 + (7 - bit)] = if (b >> bit) & 1 == 1 { 130 } else { 65 };
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::at32f405::Peripherals::take().unwrap();
    
    // 1. Initialize Clocks to 216MHz
    hw::init_clocks(&dp.CRM, &dp.FLASH);

    // 2. Initialize Hardware
    hw::init_adc_dma(&dp, core::ptr::addr_of_mut!(ADC_BUFFER) as u32, NUM_KEYS as u16);
    hw::init_rgb(&dp, unsafe { core::ptr::addr_of_mut!(RGB_BUFFER) as u32 }, RGB_BUF_LEN as u16);

    // 3. Initialize Key Logic State
    let mut keys = [HallKey::new(); NUM_KEYS];
    let config = he_logic::KeyConfig {
        actuation_mm: 12,        // 1.2mm
        release_mm: 10,          // 1.0mm
        rt_enabled: true,
        rt_press_mm: 1,          // 0.1mm sensitivity
        rt_release_mm: 1,        // 0.1mm sensitivity
        top_deadzone_mm: 20,     // 0.2mm (Prevents RT near top)
        bottom_deadzone_mm: 380,  // 3.8mm (Prevents RT near bottom)
    };

    // --- GUIDED CALIBRATION ---
    // We calibrate keys one by one. You can use RGB LEDs to guide the user.
    for i in 0..NUM_KEYS {
        // 1. Set baseline for the current key while it's up
        let sample = unsafe { *core::ptr::addr_of!(ADC_BUFFER[i]) };
        keys[i].set_baseline(sample);

        // 2. Wait for this specific key to be calibrated
        while keys[i].state != he_logic::KeyState::Ready {
            let sample = unsafe { *core::ptr::addr_of!(ADC_BUFFER[i]) };
            keys[i].update(sample, &config);

            // RGB UI Feedback
            match keys[i].state {
                he_logic::KeyState::WaitingForDiscovery => set_led(i, 0, 0, 100), // Dim Blue
                he_logic::KeyState::Discovering => set_led(i, 100, 100, 0),       // Yellow
                _ => set_led(i, 0, 100, 0),                                       // Green
            }
            hw::update_rgb(&dp.DMA1, RGB_BUF_LEN as u16);

            cortex_m::asm::delay(1000); // 1ms-ish
        }
    }

    // 4. Initialize USB HS (Target 8kHz)
    let usb_peripheral = hw::OtghsPeripheral {
        global: dp.USB_OTGHS_GLOBAL,
        device: dp.USB_OTGHS_DEVICE,
        pwrclk: dp.USB_OTGHS_PWRCLK,
    };

    static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<synopsys_usb_otg::UsbBus<hw::OtghsPeripheral>>> = None;
    unsafe {
        USB_BUS = Some(synopsys_usb_otg::UsbBus::new(usb_peripheral, &mut [0u32; 1024]));
    }
    let usb_bus = unsafe { USB_BUS.as_ref().unwrap() };

    let mut hid = usbd_hid::hid_class::HIDClass::new(usb_bus, &[
        0x05, 0x01,        // Usage Page (Generic Desktop)
        0x09, 0x06,        // Usage (Keyboard)
        0xA1, 0x01,        // Collection (Application)
        0x05, 0x07,        //   Usage Page (Key Codes)
        0x19, 0xE0,        //   Usage Minimum (224)
        0x29, 0xE7,        //   Usage Maximum (231)
        0x15, 0x00,        //   Logical Minimum (0)
        0x25, 0x01,        //   Logical Maximum (1)
        0x75, 0x01,        //   Report Size (1)
        0x95, 0x08,        //   Report Count (8)
        0x81, 0x02,        //   Input (Data, Variable, Absolute) ; Modifier byte
        0x95, 0x01,        //   Report Count (1)
        0x75, 0x08,        //   Report Size (8)
        0x81, 0x01,        //   Input (Constant) ; Reserved byte
        0x05, 0x07,        //   Usage Page (Key Codes)
        0x19, 0x00,        //   Usage Minimum (0)
        0x29, 0x7F,        //   Usage Maximum (127)
        0x15, 0x00,        //   Logical Minimum (0)
        0x25, 0x01,        //   Logical Maximum (1)
        0x75, 0x01,        //   Report Size (1)
        0x95, 0x80,        //   Report Count (128)
        0x81, 0x02,        //   Input (Data, Variable, Absolute) ; 128-bit bitmap
        0xC0               // End Collection
    ], 1); // bInterval = 1 (125us for High Speed)

    let mut usb_dev = usb_device::device::UsbDeviceBuilder::new(usb_bus, usb_device::device::UsbVidPid(0x1209, 0x0001))
        .manufacturer("Antigravity")
        .product("HE Keyboard")
        .serial_number("8KHZ")
        .device_class(0x03) // HID
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut hid]) {
            continue;
        }

        // NKRO Report: 1 byte modifiers, 1 byte reserved, 16 bytes key bitmask (128 bits)
        let mut report = [0u8; 18]; 

        for i in 0..NUM_KEYS {
            let raw_sample = unsafe { *core::ptr::addr_of!(ADC_BUFFER[i]) };
            
            // Update Hall Effect logic
            keys[i].update(raw_sample, &config);

            if keys[i].is_pressed {
                let keycode = match i {
                    0 => 0x14, // Q
                    1 => 0x1A, // W
                    2 => 0x08, // E
                    3 => 0x15, // R
                    _ => 0,
                };
                
                if keycode != 0 {
                    let byte_idx = 2 + (keycode / 8) as usize;
                    let bit_idx = (keycode % 8) as u8;
                    if byte_idx < 18 {
                        report[byte_idx] |= 1 << bit_idx;
                    }
                }
            }
        }

        // Send report every 125us
        let _ = hid.push_raw_input(&report);
    }
}
