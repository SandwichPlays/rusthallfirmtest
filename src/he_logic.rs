#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DiscoveryState {
    Idle,
    WaitRelease,
    Done,
}

#[derive(Copy, Clone)]
pub struct KeyConfig {
    pub actuation_mm: u16,    // e.g. 150 for 1.5mm
    pub rt_down_mm: u16,      // e.g. 10 for 0.1mm
    pub rt_up_mm: u16,        // e.g. 10 for 0.1mm
    pub deadzone_top: u16,    // e.g. 20 for 0.2mm
    pub deadzone_bottom: u16, // e.g. 20 for 0.2mm
}

#[derive(Copy, Clone)]
pub struct HallKey {
    pub discovery: DiscoveryState,
    pub baseline: u16,
    pub max_travel: u16,
    
    pub pos_mm: u16,          // 0 to 400 (4.0mm)
    pub is_pressed: bool,
    
    pub peak_mm: u16,
    pub valley_mm: u16,
    pub last_raw: u16,
}

impl HallKey {
    pub const fn new() -> Self {
        Self {
            discovery: DiscoveryState::Idle,
            baseline: 0,
            max_travel: 0,
            pos_mm: 0,
            is_pressed: false,
            peak_mm: 0,
            valley_mm: 400,
            last_raw: 0,
        }
    }

    pub fn discovery_tick(&mut self, raw: u16) -> DiscoveryState {
        // Simple 4-tap EMA for discovery
        self.last_raw = (((self.last_raw as u32 * 3) + raw as u32) / 4) as u16;
        
        match self.discovery {
            DiscoveryState::Idle => {
                if self.baseline == 0 { self.baseline = raw; }
                // Start discovery when pressed 1mm equivalent (approx 300 raw units)
                if raw > self.baseline + 300 {
                    self.max_travel = raw;
                    self.discovery = DiscoveryState::WaitRelease;
                }
            }
            DiscoveryState::WaitRelease => {
                if raw > self.max_travel { self.max_travel = raw; }
                // Done when released back to baseline
                if raw < self.baseline + 50 {
                    self.discovery = DiscoveryState::Done;
                }
            }
            DiscoveryState::Done => {}
        }
        self.discovery
    }

    pub fn discovery_state(&self) -> DiscoveryState {
        self.discovery
    }

    pub fn tick(&mut self, raw: u16, config: &KeyConfig) -> bool {
        // 8-tap EMA for production filtering
        self.last_raw = (((self.last_raw as u32 * 7) + raw as u32) / 8) as u16;
        
        // 1. Calculate position in mm (0-400)
        let range = self.max_travel.saturating_sub(self.baseline);
        if range < 200 { return false; } // Safety
        
        let raw_pos = self.last_raw.saturating_sub(self.baseline);
        let mm = ((raw_pos as u32 * 400) / range as u32) as u16;
        self.pos_mm = if mm > 400 { 400 } else { mm };
        
        let pos = self.pos_mm;

        // 2. Rapid Trigger Logic
        if !self.is_pressed {
            // Actuation
            if pos >= config.actuation_mm {
                self.is_pressed = true;
                self.peak_mm = pos;
            } else if pos > config.deadzone_top && pos < (400 - config.deadzone_bottom) {
                if pos > (self.valley_mm.saturating_add(config.rt_down_mm)) {
                    self.is_pressed = true;
                    self.peak_mm = pos;
                }
            }
            if pos < self.valley_mm { self.valley_mm = pos; }
        } else {
            // Release
            if pos <= config.deadzone_top {
                self.is_pressed = false;
                self.valley_mm = pos;
            } else if pos < (self.peak_mm.saturating_sub(config.rt_up_mm)) {
                self.is_pressed = false;
                self.valley_mm = pos;
            }
            if pos > self.peak_mm { self.peak_mm = pos; }
        }

        self.is_pressed
    }
}

pub static KEY_MAP: [u8; 64] = [
    0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25,
    0x26, 0x27, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x14, 0x1A, 0x08, 0x15, 0x17, 0x1C, 0x18, 0x0C,
    0x12, 0x13, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34,
    0x04, 0x16, 0x07, 0x09, 0x0A, 0x0B, 0x0D, 0x0E,
    0x0F, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x28,
    0x1D, 0x1B, 0x06, 0x19, 0x05, 0x11, 0x10, 0x36,
    0xE1, 0xE0, 0xE2, 0x2C, 0xE6, 0x65, 0x35, 0x2A,
];
