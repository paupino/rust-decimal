use crate::decimal::{CalculationResult, Decimal, MAX_PRECISION, POWERS_10};

// The maximum power of 10 that a 32 bit integer can store
pub const MAX_I32_SCALE: u32 = 9;
// The maximum power of 10 that a 64 bit integer can store
pub const MAX_I64_SCALE: u32 = 19;

// TODO: Test out array
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

// TODO: Test out array
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
}

#[derive(Debug)]
pub struct Buf24 {
    pub data: [u32; 6],
}

impl Buf24 {
    pub const fn zero() -> Self {
        Buf24 {
            data: [0, 0, 0, 0, 0, 0],
        }
    }

    pub const fn low64(&self) -> u64 {
        ((self.data[1] as u64) << 32) | (self.data[0] as u64)
    }

    pub fn set_low64(&mut self, value: u64) {
        self.data[1] = (value >> 32) as u32;
        self.data[0] = value as u32;
    }

    pub const fn mid64(&self) -> u64 {
        ((self.data[3] as u64) << 32) | (self.data[2] as u64)
    }

    pub fn set_mid64(&mut self, value: u64) {
        self.data[3] = (value >> 32) as u32;
        self.data[2] = value as u32;
    }

    pub const fn high64(&self) -> u64 {
        ((self.data[5] as u64) << 32) | (self.data[4] as u64)
    }

    pub fn set_high64(&mut self, value: u64) {
        self.data[5] = (value >> 32) as u32;
        self.data[4] = value as u32;
    }

    pub const fn upper_word(&self) -> usize {
        if self.data[5] > 0 {
            return 5;
        }
        if self.data[4] > 0 {
            return 4;
        }
        if self.data[3] > 0 {
            return 3;
        }
        if self.data[2] > 0 {
            return 2;
        }
        if self.data[1] > 0 {
            return 1;
        }
        return 0;
    }

    // Attempt to rescale the number into 96 bits. If successful, the scale is returned wrapped
    // in an Option. If it failed due to overflow, we return None.
    // * `upper` - Index of last non-zero value in self.
    // * `scale` - Current scale factor for this value.
    pub fn rescale(&mut self, upper: usize, scale: u32) -> Option<u32> {
        let mut scale = scale;
        let mut upper = upper;

        // Determine a rescale target to start with
        let mut rescale_target = 0;
        if upper > 2 {
            rescale_target = upper as u32 * 32 - 64 - 1;
            rescale_target -= self.data[upper].leading_zeros();
            rescale_target = ((rescale_target * 77) >> 8) + 1;
            if rescale_target > scale {
                return None;
            }
        }

        // Make sure we scale enough to bring it into a valid range
        if scale > MAX_PRECISION && rescale_target < scale - MAX_PRECISION {
            rescale_target = scale - MAX_PRECISION;
        }

        if rescale_target > 0 {
            // We're going to keep reducing by powers of 10. So, start by reducing the scale by
            // that amount.
            scale = scale - rescale_target;
            let mut sticky = 0;
            let mut remainder = 0;
            loop {
                sticky = sticky | remainder;
                let mut power = if rescale_target > 8 {
                    POWERS_10[9]
                } else {
                    POWERS_10[rescale_target as usize]
                };

                let high = self.data[upper];
                let high_quotient = high / power;
                remainder = high - high_quotient * power;

                for item in self.data.iter_mut().rev().skip(6 - upper) {
                    let num = (*item as u64).wrapping_add((remainder as u64) << 32);
                    *item = (num / power as u64) as u32;
                    remainder = (num as u32).wrapping_sub((*item).wrapping_mul(power));
                }

                self.data[upper] = high_quotient;

                // If the high quotient was zero then decrease the upper bound
                if high_quotient == 0 && upper > 0 {
                    upper -= 1;
                }
                if rescale_target > MAX_I32_SCALE {
                    // Scale some more
                    rescale_target -= MAX_I32_SCALE;
                    continue;
                }

                // If we fit into 96 bits then we've scaled enough. Otherwise, scale once more.
                if upper > 2 {
                    if scale == 0 {
                        return None;
                    }
                    // Equivalent to scaling down by 10
                    rescale_target = 1;
                    scale -= 1;
                    continue;
                }

                // Round the final result.
                power = power >> 1;
                let carried = if power <= remainder {
                    // If we're less than half then we're fine. Otherwise, we round if odd or if the
                    // sticky bit is set.
                    if power < remainder || ((self.data[0] & 1) | sticky) != 0 {
                        // Round up
                        self.data[0] = self.data[0].wrapping_add(1);
                        // Check if we carried
                        self.data[0] == 0
                    } else {
                        false
                    }
                } else {
                    false
                };

                // If we carried then propagate through the portions
                if carried {
                    let mut pos = 0;
                    for (index, value) in self.data.iter_mut().enumerate().skip(1) {
                        pos = index;
                        *value = value.wrapping_add(1);
                        if *value != 0 {
                            break;
                        }
                    }

                    // If we ended up rounding over the 96 bits then we'll try to rescale down (again)
                    if pos > 2 {
                        // Nothing to scale down from will cause overflow
                        if scale == 0 {
                            return None;
                        }

                        // Loop back around using scale of 10.
                        // Reset the sticky bit and remainder before looping.
                        upper = pos;
                        sticky = 0;
                        remainder = 0;
                        rescale_target = 1;
                        scale -= 1;
                        continue;
                    }
                }
                break;
            }
        }

        Some(scale)
    }
}
