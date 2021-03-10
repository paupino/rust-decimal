impl Decimal {
    fn low64(&self) -> u64 {
        ((self.mid as u64) << 32) | (self.lo as u64)
    }

    fn set_low64(&mut self, value: u64) {
        self.mid = (value >> 32) as u32;
        self.lo = value as u32;
    }
}

pub(crate) fn add_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
    add_sub_internal(d1, d2, false)
}

#[inline]
fn add_sub_internal(d1: &Decimal, d2: &Decimal, sign: bool) -> CalculationResult {
    let dec1 = Dec12::new(&d1);
    let dec2 = Dec12::new(&d2);
    let xor_flags = d1.flags ^ d2.flags;
    let sign = sign ^ ((xor_flags & SIGN_MASK) != 0);

    // If the scale of the XORd flags is 0 then that indicates that the scale is the same.
    if xor_flags & SCALE_MASK == 0 {
        return aligned_add(&dec1, &dec2, d1.flags, sign);
    }

    // Since the scales are different, we effectively need to rescale the number with the lower
    // scale up to the scale of the other number (if we can). The result should equal the greater
    // of the two scales. We naturally assume that the larger scale is the smaller number, however
    // this may not always be true.
    unimplemented!("add")
}

fn aligned_add(d1: &Dec12, d2: &Dec12, flags: u32, sign: bool) -> CalculationResult {
    let d1_low64 = d1.low64();
    let d1_hi = d1.hi;

    // May want to consider extending Decimal
    let mut result = Decimal {
        lo: d1.lo,
        mid: d1.mid,
        hi: d1.hi,
        flags,
    };

    if sign {
        // Signs differ meaning we need to subtract
        let low64 = d1_low64.wrapping_sub(d2.low64());
        result.set_low64(low64);
        result.hi = d1_hi.wrapping_sub(d2.hi);

        // Propagate the carry. Wrapping sub would cause low64 to be greater than d1_low64
        if low64 > d1_low64 {
            result.hi -= 1;
            if result.hi >= d1_hi {
                flip_sign(&mut result);
            }
        } else if result.hi > d1_hi {
            flip_sign(&mut result);
        }
    } else {
        // Signs are the same meaning we need to add
        let low64 = d1_low64.wrapping_add(d2.low64());
        result.set_low64(low64);
        result.hi = d1_hi.wrapping_add(d2.hi);

        // Propagate the carry. Wrapping add would cause low64 to be less than d1_low64
        if low64 < d1_low64 {
            result.hi += 1;
            if result.hi <= d1_hi {
                // The addition carried above 96 bits. Try to reduce scale factor.
                if descale(&mut result) {
                    return CalculationResult::Overflow;
                }
            }
        } else if result.hi < d1_hi {
            // The addition carried above 96 bits. Try to reduce scale factor.
            if descale(&mut result) {
                return CalculationResult::Overflow;
            }
        }
    }

    CalculationResult::Ok(result)
}

fn flip_sign(result: &mut Decimal) {
    // Flip the sign mask
    result.flags ^= SIGN_MASK;
    // Since we detected this by overflow, we also need to clean up
    // the components to take into account this negation.
    result.hi = !result.hi;
    let low64 = (-(result.low64() as i64)) as u64;
    if low64 == 0 {
        result.hi += 1;
    }
    result.set_low64(low64);
}

fn descale(result: &mut Decimal) -> bool {
    // This function attempts to reduce the scale by dividing by 10
    // If the scale is already zero then we can't reduce it anymore. It's an overflow.
    if (result.flags & SCALE_MASK) == 0 {
        return true;
    }

    // Reduce the scale by one
    result.flags -= (1 << SCALE_SHIFT);

    // Divide by 10
    let mut temp = Dec12::new(&result);
    let remainder = temp.div32(10);

    // See if we need to round up.
    if remainder >= 5 && (remainder > 5 || (temp.lo & 1) != 0) {
        let low64 = temp.low64().wrapping_add(1);
        temp.set_low64(low64);
        if low64 == 0 {
            temp.hi += 1;
        }
    }

    // Build the result
    result.lo = temp.lo;
    result.mid = temp.mid;
    result.hi = temp.hi;

    // No overflow, return false.
    false
}
