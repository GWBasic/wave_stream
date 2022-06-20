use std::io::{ Write, Result };

pub trait WriteEx : Write {
    fn write_str(&mut self, s: &str) -> Result<()>;
    fn write_u32(&mut self, v: u32) -> Result<()>;
    fn write_u16(&mut self, v: u16) -> Result<()>;
    fn write_f32(&mut self, v: f32) -> Result<()>;
}

impl<T> WriteEx for T where T: Write {
    fn write_str(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        self.write(&bytes)?;

        Ok(())
    }

    fn write_u32(&mut self, v: u32) -> Result<()> {
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
}
