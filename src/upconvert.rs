use std::io::{ Result };

use crate::assertions::assert_int_24;
use crate::constants::{ INT_24_ADD_FOR_FLOAT_ABS, INT_24_DIVIDE_FOR_FLOAT };

pub fn int_24_to_float(sample_int_24: i32) -> Result<f32> {
    assert_int_24(sample_int_24)?;

    let sample_int_24_as_float = sample_int_24 as f32;
    let sample_int_24_abs = sample_int_24_as_float + INT_24_ADD_FOR_FLOAT_ABS;
    Ok((sample_int_24_abs / INT_24_DIVIDE_FOR_FLOAT) - 1.0)
}


#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::constants::{ MAX_INT_24, MIN_INT_24 };

    use super::*;

    #[test_case(MAX_INT_24, 1.0; "int_24_max")]
    #[test_case(MIN_INT_24, -1.0; "int_24_min")]
    #[test_case(MAX_INT_24 / 2, 0.5; "int_24_half")]
    #[test_case((MIN_INT_24 / 2) - 1, -0.5000001; "int_24_half_negative")]
    #[test_case(MAX_INT_24 / 4, 0.25; "int_24_quarter")]
    #[test_case((MIN_INT_24 / 4) - 1, -0.25000006; "int_24_quarter_negative")]
    #[test_case(0, 1.1920929e-7; "int_24_smallest_positive")]
    #[test_case(-1, -5.9604645e-8; "int_24_smallest_negative")]
    fn int_24_to_float_test(sample_int_24: i32, expected_sample_float: f32) {
        let actual_sample_float = int_24_to_float(sample_int_24).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }
}