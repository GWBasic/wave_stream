use std::io::{ Error, ErrorKind, Read, Result };
use std::str;

use crate::upconvert::int_24_to_float;

pub trait ReadEx : Read {
    fn skip(&mut self, length: usize) -> Result<()>;
    fn read_fixed_size(&mut self, buf: &mut [u8]) -> Result<()>;
    fn read_str(&mut self, len: usize) -> Result<String>;
    fn assert_str(&mut self, expected: &str, error_kind: ErrorKind, message: &str) -> Result<()>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_i8(&mut self) -> Result<i8>;
    fn read_i24(&mut self) -> Result<i32>;
    fn read_i24_as_float(&mut self) -> Result<f32>;
}

impl<T> ReadEx for T where T: Read {
    fn skip(&mut self, length: usize) -> Result<()> {
        if length > 0 {
            let mut buf = [0u8];
            for _ in 0..length {
                self.read_exact(&mut buf[..])?;
            }
        }

        Ok(())
    }

    fn read_fixed_size(&mut self, buf: &mut [u8]) -> Result<()> {
        let bytes_read = self.read(buf)?;
    
        if bytes_read == buf.len() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::UnexpectedEof, "Unexpected end of file"))
        }
    }

    fn read_str(&mut self, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len];
        self.read_fixed_size(&mut buf[..])?;
    
        match String::from_utf8(buf) {
            Ok(s) => Ok(s),
            Err(utf8error) => Err(Error::new(ErrorKind::Other, format!("{}", utf8error)))
        }
    }

    fn assert_str(&mut self, expected: &str, error_kind: ErrorKind, message: &str) -> Result<()> {
        let actual = self.read_str(expected.len())?;

        if expected.eq(&actual) {
            Ok(())
        } else {
            Err(Error::new(error_kind, message))
        }    
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_fixed_size(&mut buf[..])?;

        Ok(u32::from_le_bytes(buf))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_fixed_size(&mut buf[..])?;

        Ok(i16::from_le_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_fixed_size(&mut buf[..])?;

        Ok(u16::from_le_bytes(buf))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_fixed_size(&mut buf[..])?;

        Ok(f32::from_le_bytes(buf))
    }

    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8; 1];
        self.read_fixed_size(&mut buf[..])?;

        Ok(i8::from_le_bytes(buf))
    }

    fn read_i24(&mut self) -> Result<i32> {
        let mut buf = [0u8; 3];
        self.read_fixed_size(&mut buf[..])?;

        let buf = [buf[0], buf[1], buf[2], 0];
        Ok(i32::from_le_bytes(buf) >> 8)
    }

    fn read_i24_as_float(&mut self) -> Result<f32> {
        let sample_int_24 = self.read_i24()?;
        return int_24_to_float(sample_int_24);
    }
}