use at32f4xx_pac as pac;

/// Initialize system clocks to 216MHz using an 8MHz external crystal (HEXT).
pub fn init_clocks(crm: &pac::at32f405::crm::RegisterBlock, flash: &pac::at32f405::flash::RegisterBlock) {
    // 1. Enable HEXT
    crm.ctrl().modify(|_, w| w.hexten().set_bit());
    while crm.ctrl().read().hextstbl().bit_is_clear() {}

    // 2. Set Flash Latency (Required for high frequency)
    flash.psr().modify(|_, w| unsafe { w.wtcyc().bits(0b111) });

    // 3. Configure PLL
    crm.pllcfg().modify(|_, w| unsafe {
        w.pllrcs().set_bit() // Select HEXT as source
         .pll_ms().bits(1)   
         .pll_ns().bits(54) 
         .pll_fp().bits(2) // pllfr -> pll_fp
    });

    // 4. Enable PLL and wait for stability
    crm.ctrl().modify(|_, w| w.pllen().set_bit());
    while crm.ctrl().read().pllstbl().bit_is_clear() {}

    // 5. Switch System Clock to PLL (2 = PLL)
    crm.cfg().modify(|_, w| unsafe { w.sclksel().bits(2) }); 
    while crm.cfg().read().sclksts().bits() != 2 {}          

    // 6. Set Bus Prescalers
    crm.cfg().modify(|_, w| unsafe {
        w.ahbdiv().bits(0)    
         .apb1div().bits(4) // Div 2
         .apb2div().bits(4) // Div 2
    });
}

/// Initialize ADC1 and DMA1 for high-speed matrix scanning.
pub fn init_adc_dma(dp: &pac::at32f405::Peripherals, dma_buffer: u32, buffer_len: u16) {
    let crm = &dp.CRM;
    let adc1 = &dp.ADC1;
    let dma1 = &dp.DMA1;

    // Enable Peripheral Clocks
    crm.apb2en().modify(|_, w| w.adc1().set_bit()); 
    crm.ahben1().modify(|_, w| w.dma1().set_bit()); 

    // --- ADC Setup ---
    dp.GPIOA.cfgr().modify(|_, w| unsafe { 
        w.iomc0().bits(3) // Analog mode
         .iomc1().bits(3)
         .iomc2().bits(3)
         .iomc3().bits(3)
    });

    // Configure Sequence (Artery: osq)
    // oslen is in osq1, defines (n-1) conversions
    adc1.osq1().modify(|_, w| unsafe { w.oclen().bits(3) }); 
    
    // First 4 channels are in osq3 (osn1..osn4 fields)
    adc1.osq3().modify(|_, w| unsafe {
        w.osn1().bits(0) // 1st conversion: CH0
         .osn2().bits(1) // 2nd conversion: CH1
         .osn3().bits(2) // 3rd conversion: CH2
         .osn4().bits(3) // 4th conversion: CH3
    });

    // Enable Scan Mode and DMA Repeat
    adc1.ctrl1().modify(|_, w| w.sqen().set_bit()); 
    adc1.ctrl2().modify(|_, w| w.ocdmaen().set_bit().rpen().set_bit()); 

    // Enable ADC and Calibration
    adc1.ctrl2().modify(|_, w| w.adcen().set_bit());
    adc1.ctrl2().modify(|_, w| w.adcal().set_bit());
    while adc1.ctrl2().read().adcal().bit_is_set() {}

    // --- DMA Setup (DMA1 Channel 1 is linked to ADC1) ---
    let channel = dma1.channel1(); // Corrected: channel1() is on dma1
    channel.paddr().write(|w| unsafe { w.bits(0x4001244C) }); // ADC1_ODT address
    channel.maddr().write(|w| unsafe { w.bits(dma_buffer) });
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });

    channel.ctrl().modify(|_, w| unsafe {
        w.dtd().clear_bit()     // Peripheral to Memory
         .lm().set_bit()        // Circular mode (Loop Mode)
         .pincm().clear_bit()   // Peripheral no increment
         .mincm().set_bit()      // Memory increment
         .pwidth().bits(1)      // 16-bit
         .mwidth().bits(1)      // 16-bit
         .chen().set_bit()      // Enable Channel
    });

    // Start ADC conversion
    adc1.ctrl2().modify(|_, w| w.ocswtrg().set_bit()); 
}

/// Initialize TMR1 and DMA1 for WS2812B RGB on PA8.
pub fn init_rgb(dp: &pac::at32f405::Peripherals, dma_buffer: u32, buffer_len: u16) {
    let crm = &dp.CRM;
    let tmr1 = &dp.TMR1;
    let dma1 = &dp.DMA1;

    // 1. Enable Clocks
    crm.apb2en().modify(|_, w| w.tmr1().set_bit().gpioa().set_bit());
    
    // 2. Configure PA8 as TMR1_CH1 (AF1)
    dp.GPIOA.cfgr().modify(|_, w| unsafe { w.iomc8().bits(2) }); // AF mode
    dp.GPIOA.muxh().modify(|_, w| unsafe { w.mux8().bits(1) });  // MUX8 = AF1

    // 3. Configure TMR1 (216MHz)
    // Target 800kHz (1.25us period) -> 216 / 0.8 = 270 cycles
    tmr1.pr().write(|w| unsafe { w.bits(269) }); 
    tmr1.div().write(|w| unsafe { w.bits(0) });

    // PWM Mode 1 on CH1
    tmr1.cm1_output().modify(|_, w| unsafe { w.c1ocm().bits(6).c1oen().set_bit() });
    tmr1.ctrl1().modify(|_, w| w.prben().set_bit()); // Buffer PR
    
    // Enable DMA request on CC1
    tmr1.iden().modify(|_, w| w.c1den().set_bit());

    // 4. Configure DMA1 Channel 2 (TMR1_CH1)
    let channel = dma1.channel2();
    channel.paddr().write(|w| unsafe { w.bits(0x40010034) }); // TMR1_C1DT address
    channel.maddr().write(|w| unsafe { w.bits(dma_buffer) });
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });

    channel.ctrl().modify(|_, w| unsafe {
        w.dtd().set_bit()       // Memory to Peripheral
         .lm().clear_bit()      // Normal mode (Send once)
         .pincm().clear_bit()   // Peripheral no increment
         .mincm().set_bit()      // Memory increment
         .pwidth().bits(1)      // 16-bit
         .mwidth().bits(1)      // 16-bit
         .chen().clear_bit()
    });

    // 5. Enable Timer
    tmr1.ctrl1().modify(|_, w| w.tmren().set_bit());
    tmr1.brk().modify(|_, w| w.oen().set_bit()); // Main output enable for advanced timers
}

pub fn update_rgb(dma1: &pac::at32f405::dma1::RegisterBlock, buffer_len: u16) {
    let channel = dma1.channel2();
    channel.ctrl().modify(|_, w| w.chen().clear_bit());
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });
    channel.ctrl().modify(|_, w| w.chen().set_bit());
}
