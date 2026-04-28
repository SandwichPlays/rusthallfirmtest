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

#[entry]
fn main() -> ! {
    let dp = pac::at32f405::Peripherals::take().unwrap();
    
    // 1. Initialize Clocks to 216MHz (HEXT 8MHz -> PLL -> SCLK)
    hw::init_clocks(&dp.CRM, &dp.FLASH);

    // 2. Initialize GPIO for Matrix/Multiplexers
    // TODO: Configure pins for Row/Column selection
    dp.CRM.apb2en.modify(|_, w| w.gpioaen().set_bit().gpioben().set_bit());
    
    // 3. Initialize ADC and DMA for high-speed circular scanning
    unsafe {
        hw::init_adc_dma(&dp, ADC_BUFFER.as_ptr() as u32, NUM_KEYS as u16);
    }

    // 4. Initialize Key Logic State
    let mut keys = [HallKey::new(); NUM_KEYS];

    // --- CALIBRATION PHASE ---
    // Take 1000 samples to find the resting baseline for each sensor
    for _ in 0..1000 {
        for i in 0..NUM_KEYS {
            let sample = unsafe { ADC_BUFFER[i] };
            keys[i].calibrate_baseline(sample);
        }
        // Small delay between samples
        cortex_m::asm::delay(1000);
    }

    // 5. Initialize USB HS (Target 8kHz)
    // TODO: Setup USB OTG HS peripheral

    loop {
        // NKRO Report: 1 byte modifiers, 1 byte reserved, 16 bytes key bitmask (128 bits)
        let mut report_mask = [0u8; 16]; 

        for i in 0..NUM_KEYS {
            let raw_sample = unsafe { ADC_BUFFER[i] };
            
            // Update Hall Effect logic
            keys[i].update(
                raw_sample, 
                keys[i].baseline + 200, 
                15, 
                15
            );

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
