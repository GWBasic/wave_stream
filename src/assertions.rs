use std::io::{ Error, ErrorKind, Result };

use crate::constants::{ MAX_INT_24, MIN_INT_24 };

// Asserts that the value is a valid 24-bit int
// (Because rust doesn't support 24-bit ints, they are put into 32-bit ints)
pub fn assert_int_24(v: i32) -> Result<()> {
    if v < MIN_INT_24 || v > MAX_INT_24 {
        return Result::Err(Error::new(ErrorKind::InvalidData, "Value must be a valid 24-bit integer"));
    }

    Ok(())
}