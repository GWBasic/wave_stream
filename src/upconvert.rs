use std::io::{ Result };

use crate::assertions::assert_int_24;

pub const INT_24_ADD_FOR_FLOAT_ABS:f32 = 8388608.0;
pub const INT_24_DIVIDE_FOR_FLOAT:f32 = 8388607.5;

pub const INT_16_ADD_FOR_FLOAT_ABS:f32 = 32768.0;
pub const INT_16_DIVIDE_FOR_FLOAT:f32 = 32767.5;

pub const INT_8_ADD_FOR_FLOAT_ABS:f32 = 128.0;
pub const INT_8_DIVIDE_FOR_FLOAT:f32 = 127.5;

pub fn i24_to_f32(sample_int_24: i32) -> Result<f32> {
    assert_int_24(sample_int_24)?;

    let sample_int_24_as_float = sample_int_24 as f32;
    let sample_int_24_abs = sample_int_24_as_float + INT_24_ADD_FOR_FLOAT_ABS;
    Ok((sample_int_24_abs / INT_24_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i16_to_f32(sample_int_16: i16) -> Result<f32> {
    let sample_int_16_as_float = sample_int_16 as f32;
    let sample_int_16_abs = sample_int_16_as_float + INT_16_ADD_FOR_FLOAT_ABS;
    Ok((sample_int_16_abs / INT_16_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i8_to_f32(sample_int_8: i8) -> Result<f32> {
    let sample_int_8_as_float = sample_int_8 as f32;
    let sample_int_8_abs = sample_int_8_as_float + INT_8_ADD_FOR_FLOAT_ABS;
    Ok((sample_int_8_abs / INT_8_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i16_to_i24(sample_int_16: i16) -> Result<i32> {
    let sample_int_32 = sample_int_16 as i32;

    let sample_int_24: i32;
    if sample_int_32 >= 0 {
        sample_int_24 = ((sample_int_32 + 1) * 256) - 1;
    } else { //sample_int_32 < 0 {
        sample_int_24 = sample_int_32 * 256;
    }

    Ok(sample_int_24)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::constants::{ MAX_INT_24, MIN_INT_24 };

    use super::*;

    #[test_case(MAX_INT_24, 1.0; "int_24_to_f32_max")]
    #[test_case(MIN_INT_24, -1.0; "int_24_to_f32_min")]
    #[test_case(MAX_INT_24 / 2, 0.5; "int_24_to_f32_half")]
    #[test_case((MIN_INT_24 / 2) - 1, -0.5000001; "int_24_to_f32_half_negative")]
    #[test_case(MAX_INT_24 / 4, 0.25; "int_24_to_f32_quarter")]
    #[test_case((MIN_INT_24 / 4) - 1, -0.25000006; "int_24_to_f32_quarter_negative")]
    #[test_case(0, 1.1920929e-7; "int_24_to_f32_smallest_positive")]
    #[test_case(-1, -5.9604645e-8; "int_24_to_f32_smallest_negative")]
    fn i24_to_f32_test(sample_int_24: i32, expected_sample_float: f32) {
        let actual_sample_float = i24_to_f32(sample_int_24).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i16::MAX, 1.0; "int_16_to_f32_max")]
    #[test_case(i16::MIN, -1.0; "int_16_to_f32_min")]
    #[test_case(i16::MAX / 2, 0.49999237; "int_16_to_f32_half")]
    #[test_case((i16::MIN / 2) - 1, -0.5000229; "int_16_to_f32_half_negative")]
    #[test_case(i16::MAX / 4, 0.24998856; "int_16_to_f32_quarter")]
    #[test_case((i16::MIN / 4) - 1, -0.25001907; "int_16_to_f32_quarter_negative")]
    #[test_case(0, 1.5258789e-5; "int_16_to_f32_smallest_positive")]
    #[test_case(-1, -1.5258789e-5; "int_16_to_f32_smallest_negative")]
    fn i16_to_f32_test(sample_int_16: i16, expected_sample_float: f32) {
        let actual_sample_float = i16_to_f32(sample_int_16).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i8::MAX, 1.0; "int_8_to_f32_max")]
    #[test_case(i8::MIN, -1.0; "int_8_to_f32_min")]
    #[test_case(i8::MAX / 2, 0.49803925; "int_8_to_f32_half")]
    #[test_case((i8::MIN / 2) - 1, -0.5058824; "int_8_to_f32_half_negative")]
    #[test_case(i8::MAX / 4, 0.24705887; "int_8_to_f32_quarter")]
    #[test_case((i8::MIN / 4) - 1, -0.25490195; "int_8_to_f32_quarter_negative")]
    #[test_case(0, 0.003921628; "int_8_to_f32_smallest_positive")]
    #[test_case(-1, -0.0039215684; "int_8_to_f32_smallest_negative")]
    fn i8_to_f32_test(sample_int_8: i8, expected_sample_float: f32) {
        let actual_sample_float = i8_to_f32(sample_int_8).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i16::MAX, MAX_INT_24; "int_16_i24_max")]
    #[test_case(i16::MIN, MIN_INT_24; "int_16_i24_min")]
    #[test_case(i16::MAX / 2, MAX_INT_24 / 2; "int_16_i24_half")]
    #[test_case(i16::MIN / 2, MIN_INT_24 / 2; "int_16_i24_half_negative")]
    #[test_case(i16::MAX / 4, MAX_INT_24 / 4; "int_16_i24_quarter")]
    #[test_case(i16::MIN / 4, MIN_INT_24 / 4; "int_16_i24_quarter_negative")]
    #[test_case(0, 255; "int_16_i24_smallest_positive")]
    #[test_case(-1, -256; "int_16_i24_smallest_negative")]
    fn i16_to_i24_test(sample_int_16: i16, expected_sample_i24: i32) {
        let actual_sample_float = i16_to_i24(sample_int_16).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_i24);
    }
}