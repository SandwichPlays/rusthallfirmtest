#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum KeyState {
    WaitingForDiscovery, // Not yet calibrated
    Discovering,         // Currently being pressed for calibration
    Ready,               // Calibration locked
}

#[derive(Copy, Clone)]
pub struct KeyConfig {
    pub actuation_mm: u16,
    pub release_mm: u16,
    pub rt_enabled: bool,
    pub rt_press_mm: u16,
    pub rt_release_mm: u16,
    pub top_deadzone_mm: u16,
    pub bottom_deadzone_mm: u16,
}

#[derive(Copy, Clone)]
pub struct HallKey {
    pub state: KeyState,
    pub filtered_value: u16,
    pub baseline: u16,
    pub max_travel: u16,
    
    pub pos_mm: u16,
    pub is_pressed: bool,
    
    pub peak_mm: u16,
    pub valley_mm: u16,
}

impl HallKey {
    pub const fn new() -> Self {
        Self {
            state: KeyState::WaitingForDiscovery,
            filtered_value: 0,
            baseline: 0,
            max_travel: 0,
            pos_mm: 0,
            is_pressed: false,
            peak_mm: 0,
            valley_mm: 400,
        }
    }

    pub fn set_baseline(&mut self, sample: u16) {
        self.baseline = sample;
        self.filtered_value = sample;
        self.max_travel = sample; // Start here
    }

    fn update_position(&mut self) {
        let range = self.max_travel.saturating_sub(self.baseline);
        if range < 100 { 
            self.pos_mm = 0;
            return;
        }

        let raw_pos = self.filtered_value.saturating_sub(self.baseline);
        let mm = ((raw_pos as u32 * 400) / range as u32) as u16;
        self.pos_mm = if mm > 400 { 400 } else { mm };
    }

    pub fn update(&mut self, new_raw: u16, config: &KeyConfig) {
        self.filtered_value = (((self.filtered_value as u32 * 7) + new_raw as u32) / 8) as u16;

        match self.state {
            KeyState::WaitingForDiscovery => {
                // If value rises significantly above baseline, start discovery
                if self.filtered_value > self.baseline + 150 {
                    self.state = KeyState::Discovering;
                }
            }
            KeyState::Discovering => {
                // Track the absolute maximum during the press
                if self.filtered_value > self.max_travel {
                    self.max_travel = self.filtered_value;
                }
                // Return to baseline (plus small buffer) means calibration for this key is done
                if self.filtered_value < self.baseline + 50 {
                    if self.max_travel > self.baseline + 300 { // Ensure they actually pressed it
                        self.state = KeyState::Ready;
                    } else {
                        self.state = KeyState::WaitingForDiscovery; // Didn't press deep enough
                    }
                }
            }
            KeyState::Ready => {
                self.update_position();
                let pos = self.pos_mm;

                if !self.is_pressed {
                    if pos >= config.actuation_mm {
                        self.is_pressed = true;
                        self.peak_mm = pos;
                    } else if config.rt_enabled && pos > config.top_deadzone_mm && pos < config.bottom_deadzone_mm {
                        if pos > (self.valley_mm + config.rt_press_mm) {
                            self.is_pressed = true;
                            self.peak_mm = pos;
                        }
                    }
                    if pos < self.valley_mm { self.valley_mm = pos; }
                } else {
                    if !config.rt_enabled && pos <= config.release_mm {
                        self.is_pressed = false;
                        self.valley_mm = pos;
                    } else if config.rt_enabled {
                        if pos >= config.bottom_deadzone_mm {
                             self.peak_mm = pos;
                        } else if pos < (self.peak_mm.saturating_sub(config.rt_release_mm)) {
                            self.is_pressed = false;
                            self.valley_mm = pos;
                        }
                    }
                    if pos <= config.top_deadzone_mm {
                        self.is_pressed = false;
                        self.valley_mm = pos;
                    }
                    if pos > self.peak_mm { self.peak_mm = pos; }
                }
            }
        }
    }
}
