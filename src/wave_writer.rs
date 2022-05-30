use std::fs::File;
use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, };
//use std::iter::IntoIterator;

use crate::WriteEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWavWriter {
    writer: File,
    header: WavHeader,
    data_start: u32,
    chunk_size_written: bool
}

impl OpenWavWriter {
    pub fn new(mut writer: File, header: WavHeader) -> Result<OpenWavWriter> {
        writer.write_str("data")?;
        writer.write_u32(0)?;

        let data_start = writer.stream_position()? as u32;

        Ok(OpenWavWriter {
            writer,
            header,
            data_start,
            chunk_size_written: false
        })
    }

    fn assert_float(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }

    pub fn get_random_access_float_writer(self) -> Result<RandomAccessWavReaderFloat> {
        self.assert_float()?;

        Ok(RandomAccessWavReaderFloat {
            open_wav: self
        })
    }

    /*
    pub fn get_stream_float_reader(self) -> Result<StreamWavReaderFloat<TWriter>> {
        self.assert_float()?;

        Ok(StreamWavReaderFloat {
            open_wav: self
        })
    }
*/

    pub fn sample_format(&self) -> SampleFormat {
        self.header.sample_format
    }

    pub fn channels(&self) -> u16 {
        self.header.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.header.sample_rate
    }

    pub fn bits_per_sample(&self) -> u16 {
        self.header.bits_per_sample
    }

    pub fn bytes_per_sample(&self) -> u16 {
        self.header.bits_per_sample / 8
    }

    pub fn cleanup(&mut self) -> Result<()> {
        let chunk_size = self.writer.metadata()?.len() as u32 - self.data_start;
        self.writer.seek(SeekFrom::Start(self.data_start as u64))?;
        self.writer.write_u32(chunk_size)?;

        self.chunk_size_written = true;
        Ok(())
    }
}

impl Drop for OpenWavWriter {
    fn drop(&mut self) {
        if !self.chunk_size_written {
            self.cleanup().unwrap();
        }
    }
}


pub trait RandomAccessWavWriter<T> {
    fn info(&self) -> &OpenWavWriter;
    fn write_sample(&mut self, sample: u32, channel: u16, value: T) -> Result<()>;
}

pub struct RandomAccessWavReaderFloat {
    open_wav: OpenWavWriter
}

impl RandomAccessWavWriter<f32> for RandomAccessWavReaderFloat {
    fn info(&self) -> &OpenWavWriter {
        &(self.open_wav)
    }

    fn write_sample(&mut self, sample: u32, channel: u16, value: f32) -> Result<()> {
        if channel >= self.open_wav.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        let sample_in_channels = (sample * self.open_wav.channels() as u32) + channel as u32;
        let sample_in_bytes = (sample_in_channels as u64) * (self.open_wav.bytes_per_sample() as u64);
        let position = (self.open_wav.data_start as u64) + sample_in_bytes;

        // Pad the file if needed
        let len = self.open_wav.writer.metadata()?.len();
        if position > len {
            let channels_to_pad = self.open_wav.channels() - (channel + 1);
            let size = position + ((channels_to_pad * self.open_wav.bytes_per_sample()) as u64);

            self.open_wav.writer.set_len(size)?;
        }

        self.open_wav.writer.seek(SeekFrom::Start(position as u64))?;
        
        self.open_wav.chunk_size_written = false;
        self.open_wav.writer.write_f32(value)
    }
}