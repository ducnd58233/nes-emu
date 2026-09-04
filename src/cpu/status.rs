#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Status(u8);

impl Status {
    pub const CARRY: u8 = 1 << 0;
    pub const ZERO: u8 = 1 << 1;
    pub const INTERRUPT_DISABLE: u8 = 1 << 2;
    pub const DECIMAL: u8 = 1 << 3;
    pub const BREAK: u8 = 1 << 4;
    pub const UNUSED: u8 = 1 << 5;
    pub const OVERFLOW: u8 = 1 << 6;
    pub const NEGATIVE: u8 = 1 << 7;

    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    pub(crate) const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub(crate) fn insert(&mut self, flag: u8) {
        self.0 |= flag;
    }

    pub(crate) fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    pub(crate) fn set(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    pub(crate) fn update_zero_and_negative(&mut self, value: u8) {
        self.set(Self::ZERO, value == 0);
        self.set(Self::NEGATIVE, value & 0x80 != 0);
    }
}

impl Default for Status {
    fn default() -> Self {
        Self(Self::INTERRUPT_DISABLE | Self::UNUSED)
    }
}
