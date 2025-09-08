use std::io;

use borsh::BorshDeserialize;

use crate::{
    Decimal, Error,
    constants::{SCALE_MASK, SCALE_SHIFT, SIGN_MASK},
};

impl borsh::BorshDeserialize for Decimal {
    /// An implementation of [`BorshDeserialize`] that checks the received data to ensure it's a
    /// valid instance of [`Self`].
    fn deserialize_reader<__R: io::Read>(reader: &mut __R) -> Result<Self, io::Error> {
        const FLAG_MASK: u32 = SCALE_MASK | SIGN_MASK;

        let flags: u32 = BorshDeserialize::deserialize_reader(reader)?;
        if flags & FLAG_MASK != flags {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid flag representation",
            ));
        }

        let negative = flags & SIGN_MASK != 0;

        let scale = (flags & SCALE_MASK) >> SCALE_SHIFT;
        if scale > Self::MAX_SCALE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                Error::ScaleExceedsMaximumPrecision(scale),
            ));
        }

        let hi = BorshDeserialize::deserialize_reader(reader)?;
        let lo = BorshDeserialize::deserialize_reader(reader)?;
        let mid = BorshDeserialize::deserialize_reader(reader)?;

        Ok(Self::from_parts(lo, mid, hi, negative, scale))
    }
}
