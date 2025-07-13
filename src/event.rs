#[derive(Debug, Clone, Copy)]
pub enum KeyPressState {
    KeyUp,
    KeyDown,
}
pub enum Event {
    Key {
        ch: char,
        modifiers: ModifierKeyCode,
        state: KeyPressState,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ModifierKeyCode(u8);

impl ModifierKeyCode {
    pub const CTRL: u8 = 0b00000001;
    pub const ALT: u8 = 0b00000010;
    pub const SHIFT: u8 = 0b00000100;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn set_ctrl(&mut self, is_pressed: bool) -> Self {
        if is_pressed {
            Self(self.0 | Self::CTRL)
        } else {
            Self(self.0 & !Self::CTRL)
        }
    }
    pub fn set_alt(&mut self, is_pressed: bool) -> Self {
        if is_pressed {
            Self(self.0 | Self::ALT)
        } else {
            Self(self.0 & !Self::ALT)
        }
    }
    pub fn set_shift(&mut self, is_pressed: bool) -> Self {
        if is_pressed {
            Self(self.0 | Self::SHIFT)
        } else {
            Self(self.0 & !Self::SHIFT)
        }
    }

    pub fn is_shift_pressed(&self) -> bool {
        return (self.0 & Self::SHIFT) > 0;
    }
    pub fn is_ctrl_pressed(&self) -> bool {
        return (self.0 & Self::CTRL) > 0;
    }
    pub fn is_alt_pressed(&self) -> bool {
        return (self.0 & Self::ALT) > 0;
    }
}
