use std::io::{ Result };

use crate::assertions::assert_int_24;

pub const INT_24_ADD_FOR_FLOAT_ABS:f32 = 8388608.0;
pub const INT_24_DIVIDE_FOR_FLOAT:f32 = 8388607.5;

pub const INT_16_ADD_FOR_FLOAT_ABS:f32 = 32768.0;
pub const INT_16_DIVIDE_FOR_FLOAT:f32 = 32767.5;

pub const INT_8_ADD_FOR_FLOAT_ABS:f32 = 128.0;
pub const INT_8_DIVIDE_FOR_FLOAT:f32 = 127.5;

pub fn i24_to_f32(sample_i24: i32) -> Result<f32> {
    assert_int_24(sample_i24)?;

    let sample_i24_as_float = sample_i24 as f32;
    let sample_i24_abs = sample_i24_as_float + INT_24_ADD_FOR_FLOAT_ABS;
    Ok((sample_i24_abs / INT_24_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i16_to_f32(sample_i16: i16) -> Result<f32> {
    let sample_i16_as_float = sample_i16 as f32;
    let sample_i16_abs = sample_i16_as_float + INT_16_ADD_FOR_FLOAT_ABS;
    Ok((sample_i16_abs / INT_16_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i8_to_f32(sample_i8: i8) -> Result<f32> {
    let sample_i8_as_float = sample_i8 as f32;
    let sample_i8_abs = sample_i8_as_float + INT_8_ADD_FOR_FLOAT_ABS;
    Ok((sample_i8_abs / INT_8_DIVIDE_FOR_FLOAT) - 1.0)
}

pub fn i16_to_i24(sample_i16: i16) -> Result<i32> {
    let sample_i32 = sample_i16 as i32;

    let sample_i24: i32;
    if sample_i32 >= 0 {
        sample_i24 = ((sample_i32 + 1) * 256) - 1;
    } else { //sample_i32 < 0 {
        sample_i24 = sample_i32 * 256;
    }

    Ok(sample_i24)
}

pub fn i8_to_i24(sample_i8: i8) -> Result<i32> {
    let sample_i32 = sample_i8 as i32;

    let sample_i24: i32;
    if sample_i32 >= 0 {
        sample_i24 = ((sample_i32 + 1) * 65536) - 1;
    } else { //sample_i32 < 0 {
        sample_i24 = sample_i32 * 65536;
    }

    Ok(sample_i24)
}

pub fn i8_to_i16(sample_i8: i8) -> Result<i16> {
    let sample_i32 = sample_i8 as i32;

    let sample_i16: i32;
    if sample_i32 >= 0 {
        sample_i16 = ((sample_i32 + 1) * 256) - 1;
    } else { //sample_i32 < 0 {
        sample_i16 = sample_i32 * 256;
    }

    Ok(sample_i16 as i16)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::constants::{ MAX_INT_24, MIN_INT_24 };

    use super::*;

    #[test_case(MAX_INT_24, 1.0; "i24_to_f32_max")]
    #[test_case(MIN_INT_24, -1.0; "i24_to_f32_min")]
    #[test_case(MAX_INT_24 / 2, 0.5; "i24_to_f32_half")]
    #[test_case((MIN_INT_24 / 2) - 1, -0.5000001; "i24_to_f32_half_negative")]
    #[test_case(MAX_INT_24 / 4, 0.25; "i24_to_f32_quarter")]
    #[test_case((MIN_INT_24 / 4) - 1, -0.25000006; "i24_to_f32_quarter_negative")]
    #[test_case(0, 1.1920929e-7; "i24_to_f32_smallest_positive")]
    #[test_case(-1, -5.9604645e-8; "i24_to_f32_smallest_negative")]
    fn i24_to_f32_test(sample_i24: i32, expected_sample_float: f32) {
        let actual_sample_float = i24_to_f32(sample_i24).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i16::MAX, 1.0; "i16_to_f32_max")]
    #[test_case(i16::MIN, -1.0; "i16_to_f32_min")]
    #[test_case(i16::MAX / 2, 0.49999237; "i16_to_f32_half")]
    #[test_case((i16::MIN / 2) - 1, -0.5000229; "i16_to_f32_half_negative")]
    #[test_case(i16::MAX / 4, 0.24998856; "i16_to_f32_quarter")]
    #[test_case((i16::MIN / 4) - 1, -0.25001907; "i16_to_f32_quarter_negative")]
    #[test_case(0, 1.5258789e-5; "i16_to_f32_smallest_positive")]
    #[test_case(-1, -1.5258789e-5; "i16_to_f32_smallest_negative")]
    fn i16_to_f32_test(sample_i16: i16, expected_sample_float: f32) {
        let actual_sample_float = i16_to_f32(sample_i16).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i8::MAX, 1.0; "i18_to_f32_max")]
    #[test_case(i8::MIN, -1.0; "i18_to_f32_min")]
    #[test_case(i8::MAX / 2, 0.49803925; "i18_to_f32_half")]
    #[test_case((i8::MIN / 2) - 1, -0.5058824; "i18_to_f32_half_negative")]
    #[test_case(i8::MAX / 4, 0.24705887; "i18_to_f32_quarter")]
    #[test_case((i8::MIN / 4) - 1, -0.25490195; "i18_to_f32_quarter_negative")]
    #[test_case(0, 0.003921628; "i18_to_f32_smallest_positive")]
    #[test_case(-1, -0.0039215684; "i18_to_f32_smallest_negative")]
    fn i8_to_f32_test(sample_i8: i8, expected_sample_float: f32) {
        let actual_sample_float = i8_to_f32(sample_i8).expect("Error converting sample to float");
        assert_eq!(actual_sample_float, expected_sample_float);
    }

    #[test_case(i16::MAX, MAX_INT_24; "i16_i24_max")]
    #[test_case(i16::MIN, MIN_INT_24; "i16_i24_min")]
    #[test_case(i16::MAX / 2, MAX_INT_24 / 2; "i16_i24_half")]
    #[test_case(i16::MIN / 2, MIN_INT_24 / 2; "i16_i24_half_negative")]
    #[test_case(i16::MAX / 4, MAX_INT_24 / 4; "i16_i24_quarter")]
    #[test_case(i16::MIN / 4, MIN_INT_24 / 4; "i16_i24_quarter_negative")]
    #[test_case(0, 255; "i16_i24_smallest_positive")]
    #[test_case(-1, -256; "i16_i24_smallest_negative")]
    fn i16_to_i24_test(sample_i16: i16, expected_sample_i24: i32) {
        let actual_sample_i24 = i16_to_i24(sample_i16).expect("Error converting sample to i24");
        assert_eq!(actual_sample_i24, expected_sample_i24);
    }

    #[test_case(i8::MAX, MAX_INT_24; "i18_i24_max")]
    #[test_case(i8::MIN, MIN_INT_24; "i18_i24_min")]
    #[test_case(i8::MAX / 2, MAX_INT_24 / 2; "i18_i24_half")]
    #[test_case(i8::MIN / 2, MIN_INT_24 / 2; "i18_i24_half_negative")]
    #[test_case(i8::MAX / 4, MAX_INT_24 / 4; "i18_i24_quarter")]
    #[test_case(i8::MIN / 4, MIN_INT_24 / 4; "i18_i24_quarter_negative")]
    #[test_case(0, 65535; "i18_i24_smallest_positive")]
    #[test_case(-1, -65536; "i18_i24_smallest_negative")]
    fn i8_to_i24_test(sample_i8: i8, expected_sample_i24: i32) {
        let actual_sample_i24 = i8_to_i24(sample_i8).expect("Error converting sample to i24");
        assert_eq!(actual_sample_i24, expected_sample_i24);
    }

    #[test_case(i8::MAX, i16::MAX; "i8_i16_max")]
    #[test_case(i8::MIN, i16::MIN; "i8_i16_min")]
    #[test_case(i8::MAX / 2, i16::MAX / 2; "i8_i16_half")]
    #[test_case(i8::MIN / 2, i16::MIN / 2; "i8_i16_half_negative")]
    #[test_case(i8::MAX / 4, i16::MAX / 4; "i8_i16_quarter")]
    #[test_case(i8::MIN / 4, i16::MIN / 4; "i8_i16_quarter_negative")]
    #[test_case(0, 255; "i8_i16_smallest_positive")]
    #[test_case(-1, -256; "i8_i16_smallest_negative")]
    fn i8_to_i16_test(sample_i8: i8, expected_sample_i16: i16) {
        let actual_sample_i16 = i8_to_i16(sample_i8).expect("Error converting sample to i16");
        assert_eq!(actual_sample_i16, expected_sample_i16);
    }
}