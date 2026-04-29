#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use at32f4xx_pac as pac;
use usb_device::prelude::*;
use usb_device::bus::UsbBusAllocator; // Missing import
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;

mod hw;
mod he_logic;

use he_logic::{HallKey, KeyConfig, KEY_MAP};

const NUM_KEYS: usize = 64;
static mut ADC_BUFFER: [u16; NUM_KEYS] = [0; NUM_KEYS];
static mut RGB_BUFFER: [u16; NUM_KEYS * 24 + 1] = [0; NUM_KEYS * 24 + 1];

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] modifiers=input;
        };
        #[packed_bits 8] reserved=input;
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0x65) = {
            #[packed_bits 128] bitmap=input;
        };
    }
)]
pub struct CustomKeyboardReport {
    pub modifiers: u8,
    pub reserved: u8,
    pub bitmap: [u8; 16],
}

#[entry]
fn main() -> ! {
    let dp = pac::at32f405::Peripherals::take().unwrap();
    
    hw::init_clocks(&dp.CRM, &dp.FLASH);
    hw::init_adc_dma(&dp, unsafe { core::ptr::addr_of_mut!(ADC_BUFFER) as u32 }, NUM_KEYS as u16);
    hw::init_rgb(&dp, unsafe { core::ptr::addr_of_mut!(RGB_BUFFER) as u32 }, (NUM_KEYS * 24 + 1) as u16);

    let usb_peripheral = hw::OtghsPeripheral {
        global: dp.USB_OTGHS_GLOBAL,
        device: dp.USB_OTGHS_DEVICE,
        pwrclk: dp.USB_OTGHS_PWRCLK,
    };

    let usb_bus = synopsys_usb_otg::UsbBus::new(usb_peripheral, unsafe { &mut *(0x20004000 as *mut [u32; 1024]) });
    let usb_bus_alloc = UsbBusAllocator::new(usb_bus);

    let mut hid = HIDClass::new(&usb_bus_alloc, CustomKeyboardReport::desc(), 1);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus_alloc, UsbVidPid(0x1209, 0x0001))
        .strings(&[usb_device::LangID::ENGLISH_US.into_descriptors()
            .manufacturer("Antigravity")
            .product("HE-8K Keyboard")
            .serial_number("0001")])
        .unwrap()
        .device_class(0)
        .build();

    let mut keys: [HallKey; NUM_KEYS] = [HallKey::new(); NUM_KEYS];
    let config = KeyConfig {
        actuation_mm: 150,    // 1.5mm
        rt_down_mm: 10,       // 0.1mm
        rt_up_mm: 10,         // 0.1mm
        deadzone_top: 20,     // 0.2mm
        deadzone_bottom: 20,  // 0.2mm
    };

    let mut current_cal_key: usize = 0;
    let mut cal_complete = false;

    loop {
        usb_dev.poll(&mut [&mut hid]);

        let adc_vals = unsafe { &ADC_BUFFER };

        if !cal_complete {
            let key = &mut keys[current_cal_key];
            let raw = adc_vals[current_cal_key];

            match key.discovery_tick(raw) {
                he_logic::DiscoveryState::Done => {
                    current_cal_key += 1;
                    if current_cal_key >= NUM_KEYS {
                        cal_complete = true;
                    }
                }
                _ => {}
            }
            
            unsafe {
                for i in 0..NUM_KEYS {
                    let color = if i < current_cal_key {
                        (0, 255, 0) // Green: Calibrated
                    } else if i == current_cal_key {
                        match keys[i].discovery_state() {
                            he_logic::DiscoveryState::WaitRelease => (255, 255, 0), // Yellow: Pressing
                            _ => (0, 0, 255), // Blue: Waiting
                        }
                    } else {
                        (0, 0, 0) // Off: Pending
                    };
                    set_led(i, color.0, color.1, color.2);
                }
            }
            hw::update_rgb(&dp.DMA1, (NUM_KEYS * 24 + 1) as u16);
        } else {
            let mut report = CustomKeyboardReport {
                modifiers: 0,
                reserved: 0,
                bitmap: [0; 16],
            };

            for i in 0..NUM_KEYS {
                if keys[i].tick(adc_vals[i], &config) {
                    let usb_code = KEY_MAP[i];
                    if usb_code >= 0xE0 && usb_code <= 0xE7 {
                        report.modifiers |= 1 << (usb_code - 0xE0);
                    } else if usb_code < 128 {
                        let byte = (usb_code >> 3) as usize;
                        let bit = (usb_code & 0x07) as u8;
                        if byte < 16 {
                            report.bitmap[byte] |= 1 << bit;
                        }
                    }
                }
            }
            let _ = hid.push_input(&report);
        }
    }
}

unsafe fn set_led(index: usize, r: u8, g: u8, b: u8) {
    let base = index * 24;
    for i in 0..8 {
        RGB_BUFFER[base + i] = if (g & (1 << (7 - i))) != 0 { 18 } else { 9 };
        RGB_BUFFER[base + 8 + i] = if (r & (1 << (7 - i))) != 0 { 18 } else { 9 };
        RGB_BUFFER[base + 16 + i] = if (b & (1 << (7 - i))) != 0 { 18 } else { 9 };
    }
}
