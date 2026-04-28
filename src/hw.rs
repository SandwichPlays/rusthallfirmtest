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
    adc1.osq1().modify(|_, w| unsafe { w.oslen().bits(3) }); 
    
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
    channel.paddr().write(|w| unsafe { w.paddr().bits(0x4001244C) }); // ADC1_ODT address
    channel.maddr().write(|w| unsafe { w.maddr().bits(dma_buffer) });
    channel.dtcnt().write(|w| unsafe { w.dtcnt().bits(buffer_len) });

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
