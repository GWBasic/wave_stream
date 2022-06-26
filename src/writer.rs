use std::io::{ Error, ErrorKind, Write, Result };

pub trait WriteEx : Write {
    fn write_str(&mut self, s: &str) -> Result<()>;
    fn write_i32(&mut self, v: i32) -> Result<()>;
    fn write_u32(&mut self, v: u32) -> Result<()>;
    fn write_i16(&mut self, v: i16) -> Result<()>;
    fn write_u16(&mut self, v: u16) -> Result<()>;
    fn write_f32(&mut self, v: f32) -> Result<()>;
    fn write_i8(&mut self, v: i8) -> Result<()>;
    fn write_i24(&mut self, v: i32) -> Result<()>;
}

impl<T> WriteEx for T where T: Write {
    fn write_str(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_i32(&mut self, v: i32) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_u32(&mut self, v: u32) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_i16(&mut self, v: i16) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_u16(&mut self, v: u16) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_f32(&mut self, v: f32) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_i8(&mut self, v: i8) -> Result<()> {
        let bytes = v.to_le_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_i24(&mut self, v: i32) -> Result<()> {
        if v < -8388608i32 || v > 8388607i32 {
            return Result::Err(Error::new(ErrorKind::InvalidData, "Value must be a valid 24-bit integer"));
        }

        let bytes = v.to_le_bytes();
        let bytes = [bytes[0], bytes[1], bytes[2]];
        self.write(&bytes)?;

        Ok(())
    }
}
