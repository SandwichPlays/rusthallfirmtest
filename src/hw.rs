use at32f4xx_pac as pac;
use synopsys_usb_otg::UsbPeripheral;

pub struct OtghsPeripheral {
    pub global: pac::at32f405::USB_OTGHS_GLOBAL,
    pub device: pac::at32f405::USB_OTGHS_DEVICE,
    pub pwrclk: pac::at32f405::USB_OTGHS_PWRCLK,
}

// Safety: OtghsPeripheral is a collection of PAC register blocks which are 
// pointer-like and safe to move/share between threads in this context.
unsafe impl Send for OtghsPeripheral {}
unsafe impl Sync for OtghsPeripheral {}

unsafe impl UsbPeripheral for OtghsPeripheral {
    const REGISTERS: *const () = pac::at32f405::USB_OTGHS_GLOBAL::ptr() as *const ();
    const FIFO_DEPTH_WORDS: usize = 1024; 
    const ENDPOINT_COUNT: usize = 6;
    const HIGH_SPEED: bool = true;

    fn enable() {
        let dp = unsafe { pac::at32f405::Peripherals::steal() };
        let crm = &dp.CRM;

        crm.ahben1().modify(|_, w| w.otghs().set_bit().gpioa().set_bit());
        
        // USBHS PHY selection (01: Internal HS PHY)
        crm.otghs().modify(|_, w| unsafe { w.usbhs_phy12_sel().bits(1) });

        // Reset the core (csftrst in Artery PAC)
        dp.USB_OTGHS_GLOBAL.grstctl().modify(|_, w| w.csftrst().set_bit());
        while dp.USB_OTGHS_GLOBAL.grstctl().read().csftrst().bit_is_set() {}
    }

    fn ahb_frequency_hz(&self) -> u32 {
        216_000_000
    }
}

pub fn init_clocks(crm: &pac::at32f405::crm::RegisterBlock, flash: &pac::at32f405::flash::RegisterBlock) {
    flash.psr().modify(|_, w| unsafe { w.wtcyc().bits(6) });
    crm.ctrl().modify(|_, w| w.hexten().set_bit());
    while crm.ctrl().read().hextstbl().bit_is_clear() {}

    crm.pllcfg().modify(|_, w| unsafe {
        w.pllrcs().set_bit()     // HEXT as source
         .pll_ms().bits(1)       // 8MHz / 1 = 8MHz (pll_ms in Artery PAC)
         .pll_ns().bits(54)      // 8MHz * 54 = 432MHz (pll_ns in Artery PAC)
         .pllfr().bits(1)        // 432MHz / 2 = 216MHz
    });

    crm.ctrl().modify(|_, w| w.pllen().set_bit());
    while crm.ctrl().read().pllstbl().bit_is_clear() {}

    crm.cfg().modify(|_, w| unsafe {
        w.ahbdiv().bits(0)       // AHB = 216MHz
         .apb1div().bits(4)      // APB1 = 54MHz
         .apb2div().bits(2)      // APB2 = 108MHz
         .sclksel().bits(2)      // SystemClock = PLL (sclksel in Artery PAC)
    });
}

pub fn init_adc_dma(dp: &pac::at32f405::Peripherals, buffer_ptr: u32, buffer_len: u16) {
    let crm = &dp.CRM;
    let adc1 = &dp.ADC1;
    let dma1 = &dp.DMA1;

    crm.ahben1().modify(|_, w| w.dma1().set_bit().gpioa().set_bit());
    crm.apb2en().modify(|_, w| w.adc1().set_bit());

    adc1.ctrl1().modify(|_, w| w.sqen().set_bit());
    // Artery ADC DMA bits
    adc1.ctrl2().modify(|_, w| w.ocdmaen().set_bit());

    adc1.osq1().modify(|_, w| unsafe { w.oclen().bits(3) }); 
    adc1.osq3().modify(|_, w| unsafe { 
        w.osn1().bits(0)
         .osn2().bits(1)
         .osn3().bits(2)
         .osn4().bits(3)
    });

    let channel = dma1.channel1();
    channel.paddr().write(|w| unsafe { w.bits(0x4001244C) });
    channel.maddr().write(|w| unsafe { w.bits(buffer_ptr) });
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });

    channel.ctrl().modify(|_, w| unsafe {
        w.dtd().clear_bit()      // Peripheral to Memory
         .lm().set_bit()         // Circular
         .mincm().set_bit()
         .pwidth().bits(1)       // 16-bit
         .mwidth().bits(1)       // 16-bit
         .chen().set_bit()
    });

    adc1.ctrl2().modify(|_, w| w.ocswtrg().set_bit()); 
}

pub fn init_rgb(dp: &pac::at32f405::Peripherals, dma_buffer: u32, buffer_len: u16) {
    let crm = &dp.CRM;
    let tmr1 = &dp.TMR1;
    let dma1 = &dp.DMA1;

    crm.ahben1().modify(|_, w| w.gpioa().set_bit());
    crm.apb2en().modify(|_, w| w.tmr1().set_bit());
    
    dp.GPIOA.cfgr().modify(|_, w| unsafe { w.iomc8().bits(2) });
    dp.GPIOA.muxh().modify(|_, w| unsafe { w.mux8().bits(1) });

    tmr1.pr().write(|w| unsafe { w.bits(269) }); 
    tmr1.div().write(|w| unsafe { w.bits(0) });

    // Output enable is often in CM1_OUTPUT as c1en or c1oen
    // The PAC help suggested c1oben, which is Buffer Enable.
    // Let's use the field directly if we can't find c1oen.
    tmr1.cm1_output().modify(|_, w| unsafe { w.c1c().bits(6) });
    // TMR output enable is usually in CTRL2 or BRK (for TMR1)
    tmr1.ctrl1().modify(|_, w| w.prben().set_bit());
    tmr1.iden().modify(|_, w| w.c1den().set_bit());

    let channel = dma1.channel2();
    channel.paddr().write(|w| unsafe { w.bits(0x40010034) });
    channel.maddr().write(|w| unsafe { w.bits(dma_buffer) });
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });

    channel.ctrl().modify(|_, w| unsafe {
        w.dtd().set_bit()
         .lm().clear_bit()
         .mincm().set_bit()
         .pwidth().bits(1)
         .mwidth().bits(1)
         .chen().clear_bit()
    });

    tmr1.ctrl1().modify(|_, w| w.tmren().set_bit());
    tmr1.brk().modify(|_, w| w.oen().set_bit());
}

pub fn update_rgb(dma1: &pac::at32f405::dma1::RegisterBlock, buffer_len: u16) {
    let channel = dma1.channel2();
    channel.ctrl().modify(|_, w| w.chen().clear_bit());
    channel.dtcnt().write(|w| unsafe { w.bits(buffer_len) });
    channel.ctrl().modify(|_, w| w.chen().set_bit());
}
