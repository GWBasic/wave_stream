use std::io::{ Write, Result };

pub trait WriteEx : Write {
    //fn read_fixed_size(&mut self, buf: &mut [u8]) -> Result<()>;
    //fn read_str(&mut self, len: usize) -> Result<String>;
    fn write_u32(&mut self, v: u32) -> Result<()>;
    fn write_u16(&mut self, v: u16) -> Result<()>;
    //fn read_f32(&mut self) -> Result<f32>;
}

impl<T> WriteEx for T where T: Write {
/*    fn read_fixed_size(&mut self, buf: &mut [u8]) -> Result<()> {
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
    }*/

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

/*    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_fixed_size(&mut buf[..])?;

        Ok(f32::from_le_bytes(buf))
    }*/
}
