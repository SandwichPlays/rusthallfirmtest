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
    // TODO: Setup USB OTG HS peripheral

    loop {
        // NKRO Report: 1 byte modifiers, 1 byte reserved, 16 bytes key bitmask (128 bits)
        let mut report_mask = [0u8; 16]; 

        for i in 0..NUM_KEYS {
            let raw_sample = unsafe { *core::ptr::addr_of!(ADC_BUFFER[i]) };
            
            // Update Hall Effect logic with deadzones and manual calibration data
            keys[i].update(raw_sample, &config);

            // NKRO Keymap: Set the bit corresponding to the keycode
            if keys[i].is_pressed {
                let keycode = match i {
                    0 => 0x1A, // W
                    1 => 0x04, // A
                    2 => 0x16, // S
                    3 => 0x07, // D
                    _ => 0,
                };
                
                if keycode != 0 {
                    let byte_idx = (keycode / 8) as usize;
                    let bit_idx = (keycode % 8) as u8;
                    if byte_idx < 16 {
                        report_mask[byte_idx] |= 1 << bit_idx;
                    }
                }
            }
        }

        // TODO: Send report_mask via USB HID NKRO endpoint
    }
}
