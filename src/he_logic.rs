#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeyDirection {
    Up,
    Down,
    Stationary,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct HallKey {
    pub filtered_value: u16,
    pub baseline: u16,
    pub max_travel: u16,
    pub is_pressed: bool,
    pub last_direction: KeyDirection,
    pub peak_value: u16,   // Highest value (most pressed) since last direction change
    pub valley_value: u16, // Lowest value (least pressed) since last direction change
}

impl HallKey {
    pub const fn new() -> Self {
        Self {
            filtered_value: 0,
            baseline: 2048, // Initial guess
            max_travel: 4095,
            is_pressed: false,
            last_direction: KeyDirection::Stationary,
            peak_value: 0,
            valley_value: 4095,
        }
    }

    /// Calibrates the baseline (unpressed) value. 
    /// Should be called while the key is guaranteed to be up.
    pub fn calibrate_baseline(&mut self, sample: u16) {
        self.baseline = sample;
        self.valley_value = sample;
        self.filtered_value = sample;
    }

    /// Updates the key state with a new ADC sample.
    /// sensitivities and points are in raw ADC units.
    pub fn update(
        &mut self,
        new_raw: u16,
        actuation_point: u16,
        rt_press_sensitivity: u16,
        rt_release_sensitivity: u16,
    ) {
        // 1. Exponential Moving Average (EMA) filtering
        // Simple alpha = 1/4 (25%)
        self.filtered_value = (((self.filtered_value as u32 * 3) + new_raw as u32) / 4) as u16;

        let val = self.filtered_value;

        // 2. Track direction and local extrema for Rapid Trigger
        if val > self.peak_value {
            self.peak_value = val;
            if self.last_direction != KeyDirection::Down {
                self.last_direction = KeyDirection::Down;
                self.valley_value = val; // Reset valley
            }
        } else if val < self.valley_value {
            self.valley_value = val;
            if self.last_direction != KeyDirection::Up {
                self.last_direction = KeyDirection::Up;
                self.peak_value = val; // Reset peak
            }
        }

        // 3. Actuation Logic
        if !self.is_pressed {
            // Standard Actuation
            if val >= actuation_point {
                self.is_pressed = true;
                self.peak_value = val;
                self.valley_value = val;
            }
            // Rapid Trigger Re-press
            else if val > (self.valley_value + rt_press_sensitivity) {
                self.is_pressed = true;
                self.peak_value = val;
            }
        } else {
            // Rapid Trigger Release
            if val < (self.peak_value - rt_release_sensitivity) {
                self.is_pressed = false;
                self.valley_value = val;
            }
            // Optional: Hard release at top
            if val < self.baseline + 20 { // Deadzone relative to baseline
                self.is_pressed = false;
            }
        }

        // 4. Dynamic Range Tracking (Auto-max)
        if val > self.max_travel {
            self.max_travel = val;
        }
    }
}
