use at32f4xx_pac as pac;

/// Initialize system clocks to 216MHz using an 8MHz external crystal (HEXT).
pub fn init_clocks(crm: &pac::crm::RegisterBlock, flash: &pac::flash::RegisterBlock) {
    // 1. Enable HEXT
    crm.ctrl.modify(|_, w| w.hexten().set_bit());
    while crm.ctrl.read().hextstbl().bit_is_clear() {}

    // 2. Set Flash Latency (Required for high frequency)
    // For 216MHz, usually 6-7 wait states are needed.
    flash.psr.modify(|_, w| unsafe { w.wtcyc().bits(0b111) });

    // 3. Configure PLL
    // Target: 216MHz from 8MHz HEXT.
    // Logic: (HEXT / MS) * NS / FR
    // (8 / 1) * 54 / 2 = 216MHz
    crm.pllcfg.modify(|_, w| unsafe {
        w.pllrcs().set_bit() // Select HEXT as source
         .pllms().bits(1)
         .pllns().bits(54)
         .pllfr().bits(2)
    });

    // 4. Enable PLL and wait for stability
    crm.ctrl.modify(|_, w| w.pllen().set_bit());
    while crm.ctrl.read().pllstbl().bit_is_clear() {}

    // 5. Switch System Clock to PLL
    crm.cfg.modify(|_, w| unsafe { w.sclk_sel().bits(0b10) });
    while crm.cfg.read().sclk_sts().bits() != 0b10 {}

    // 6. Set Bus Prescalers (APB1 max 120MHz, APB2 max 120MHz)
    crm.cfg.modify(|_, w| unsafe {
        w.ahbdiv().bits(0)    // AHB = 216MHz (div 1)
         .apb1div().bits(0b100) // APB1 = 108MHz (div 2)
         .apb2div().bits(0b100) // APB2 = 108MHz (div 2)
    });
}

/// Initialize ADC1 and DMA1 for high-speed matrix scanning.
pub fn init_adc_dma(dp: &pac::Peripherals, dma_buffer: u32, buffer_len: u16) {
    let crm = &dp.CRM;
    let adc1 = &dp.ADC1;
    let dma1 = &dp.DMA1;

    // Enable Peripheral Clocks
    crm.apb2en.modify(|_, w| w.adc1en().set_bit());
    crm.ahben.modify(|_, w| w.dma1en().set_bit());

    // --- ADC Setup ---
    // Configure PA0, PA1, PA2, PA3 as Analog inputs
    dp.GPIOA.omoder.modify(|_, w| unsafe {
        w.omoder0().bits(0b11) // Analog
         .omoder1().bits(0b11)
         .omoder2().bits(0b11)
         .omoder3().bits(0b11)
    });

    // Configure Sequence: PA0 (CH0), PA1 (CH1), PA2 (CH2), PA3 (CH3)
    adc1.rspt1.modify(|_, w| unsafe {
        w.rspt1().bits(0) // CH0
    });
    adc1.rspt2.modify(|_, w| unsafe {
        w.rspt2().bits(1 | (2 << 5) | (3 << 10)) // CH1, CH2, CH3
    });
    adc1.rspt_len.modify(|_, w| unsafe { w.rspt_len().bits(3) }); // 4 channels total

    // Enable Scan Mode and DMA
    adc1.ctrl1.modify(|_, w| w.scnen().set_bit());
    adc1.ctrl2.modify(|_, w| w.dmaen().set_bit().ccon().set_bit());

    // Enable ADC
    adc1.ctrl2.modify(|_, w| w.adcen().set_bit());

    // Calibration
    adc1.ctrl2.modify(|_, w| w.adcal().set_bit());
    while adc1.ctrl2.read().adcal().bit_is_set() {}

    // --- DMA Setup (DMA1 Channel 1 is typically linked to ADC1) ---
    let channel = &dma1.ch1;
    channel.paddr.write(|w| unsafe { w.bits(0x4001244C) }); // ADC1_ODT address
    channel.maddr.write(|w| unsafe { w.bits(dma_buffer) });
    channel.dtcnt.write(|w| unsafe { w.bits(buffer_len as u32) });

    channel.ctrl.modify(|_, w| unsafe {
        w.dtd().clear_bit()     // Peripheral to Memory
         .circ().set_bit()      // Circular mode
         .pinc().clear_bit()    // Peripheral no increment
         .minc().set_bit()      // Memory increment
         .psze().bits(0b01)     // 16-bit
         .msze().bits(0b01)     // 16-bit
         .chen().set_bit()      // Enable Channel
    });

    // Start ADC conversion
    adc1.ctrl2.modify(|_, w| w.adswtrg().set_bit());
}
