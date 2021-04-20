use crate::decimal::Decimal;

// The maximum power of 10 that a 32 bit integer can store
pub const MAX_I32_SCALE: u32 = 9;
// The maximum power of 10 that a 64 bit integer can store
pub const MAX_I64_SCALE: u32 = 19;

pub struct Buf12 {
    pub u0: u32,
    pub u1: u32,
    pub u2: u32,
}

impl Buf12 {
    pub const fn zero() -> Self {
        Buf12 { u0: 0, u1: 0, u2: 0 }
    }

    pub const fn low64(&self) -> u64 {
        ((self.u1 as u64) << 32) | (self.u0 as u64)
    }

    pub fn set_low64(&mut self, value: u64) {
        self.u1 = (value >> 32) as u32;
        self.u0 = value as u32;
    }

    pub const fn high64(&self) -> u64 {
        ((self.u2 as u64) << 32) | (self.u1 as u64)
    }

    pub fn set_high64(&mut self, value: u64) {
        self.u2 = (value >> 32) as u32;
        self.u1 = value as u32;
    }
}

pub struct Buf16 {
    pub u0: u32,
    pub u1: u32,
    pub u2: u32,
    pub u3: u32,
}

impl Buf16 {
    pub const fn zero() -> Self {
        Buf16 {
            u0: 0,
            u1: 0,
            u2: 0,
            u3: 0,
        }
    }

    pub const fn low64(&self) -> u64 {
        ((self.u1 as u64) << 32) | (self.u0 as u64)
    }

    pub fn set_low64(&mut self, value: u64) {
        self.u1 = (value >> 32) as u32;
        self.u0 = value as u32;
    }

    pub const fn mid64(&self) -> u64 {
        ((self.u2 as u64) << 32) | (self.u1 as u64)
    }

    pub fn set_mid64(&mut self, value: u64) {
        self.u2 = (value >> 32) as u32;
        self.u1 = value as u32;
    }

    pub const fn high64(&self) -> u64 {
        ((self.u3 as u64) << 32) | (self.u2 as u64)
    }

    pub fn set_high64(&mut self, value: u64) {
        self.u3 = (value >> 32) as u32;
        self.u2 = value as u32;
    }

    pub const fn into_buf12(&self) -> Buf12 {
        Buf12 {
            u0: self.u0,
            u1: self.u1,
            u2: self.u2,
        }
    }
}

pub struct Buf24 {
    pub u0: u32,
    pub u1: u32,
    pub u2: u32,
    pub u3: u32,
    pub u4: u32,
    pub u5: u32,
}

impl Buf24 {
    pub const fn zero() -> Self {
        Buf24 {
            u0: 0,
            u1: 0,
            u2: 0,
            u3: 0,
            u4: 0,
            u5: 0,
        }
    }

    pub const fn low64(&self) -> u64 {
        ((self.u1 as u64) << 32) | (self.u0 as u64)
    }

    pub fn set_low64(&mut self, value: u64) {
        self.u1 = (value >> 32) as u32;
        self.u0 = value as u32;
    }

    pub const fn mid64(&self) -> u64 {
        ((self.u3 as u64) << 32) | (self.u2 as u64)
    }

    pub fn set_mid64(&mut self, value: u64) {
        self.u3 = (value >> 32) as u32;
        self.u2 = value as u32;
    }

    pub const fn high64(&self) -> u64 {
        ((self.u5 as u64) << 32) | (self.u4 as u64)
    }

    pub fn set_high64(&mut self, value: u64) {
        self.u5 = (value >> 32) as u32;
        self.u4 = value as u32;
    }

    pub const fn upper_word(&self) -> u32 {
        if self.u5 != 0 {
            return 5;
        }
        if self.u4 != 0 {
            return 4;
        }
        if self.u3 != 0 {
            return 3;
        }
        if self.u2 != 0 {
            return 2;
        }
        if self.u1 != 0 {
            return 1;
        }
        return 0;
    }

    // Attempt to rescale the number into 96 bits. If successful, the scale is returned wrapped
    // in an Option. If it failed due to overflow, we return None.
    // * `upper` - Index of last non-zero value in self.
    // * `scale` - Scale factor for this value.
    pub fn rescale(&mut self, upper: u32, scale: u32) -> Option<u32> {
        // TODO: Implement
        Some(scale)
    }
}
