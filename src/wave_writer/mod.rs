use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, Write };

use crate::open_wav::OpenWav;
use crate::open_wav::OpenWavWithLength;
use crate::WriteEx;
use crate::SampleFormat;
use crate::WavHeader;

pub trait WriteSeek : Write + Seek {}

impl<TWriteSeek: Write + Seek> WriteSeek for TWriteSeek {}

pub struct OpenWavWriter {
    writer: Box<dyn WriteSeek>,
    header: WavHeader,
    data_start: u32,
    chunk_size_written: bool,
    samples_written: u32
}

pub struct RandomAccessWavWriter<T> {
    open_wav: OpenWavWriter,
    write_sample_to_stream: Box<dyn Fn(&mut dyn Write, T) -> Result<()>>
}

impl OpenWavWriter {
    pub fn new<TWriter: 'static + WriteSeek>(mut writer: TWriter, header: WavHeader) -> Result<OpenWavWriter> {
        writer.write_str("data")?;
        writer.write_u32(0)?;

        let data_start = writer.stream_position()? as u32;

        Ok(OpenWavWriter {
            writer: Box::new(writer),
            header,
            data_start,
            chunk_size_written: false,
            samples_written: 0
        })
    }

    fn assert_float(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        // data chunk
        let chunk_size = self.samples_written * (self.channels() * self.bytes_per_sample()) as u32;
        self.writer.seek(SeekFrom::Start(self.data_start as u64 - 4u64))?;
        self.writer.write_u32(chunk_size)?;

        // RIFF header
        self.writer.seek(SeekFrom::Start(4))?;
        self.writer.write_u32(chunk_size + 32 - 8)?;

        self.chunk_size_written = true;

        self.writer.flush()?;

        Ok(())
    }
}

impl OpenWav for OpenWavWriter {
    fn sample_format(&self) -> SampleFormat {
        self.header.sample_format
    }

    fn channels(&self) -> u16 {
        self.header.channels
    }

    fn sample_rate(&self) -> u32 {
        self.header.sample_rate
    }

    fn bits_per_sample(&self) -> u16 {
        self.bytes_per_sample() * 8
    }

    fn bytes_per_sample(&self) -> u16 {
        match self.header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat::Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1
        }
    }
}

impl OpenWavWithLength for OpenWavWriter {
    fn len_samples(&self) -> u32 {
        self.samples_written
    }
}

impl Drop for OpenWavWriter {
    fn drop(&mut self) {
        if !self.chunk_size_written {
            self.flush().unwrap();
        }
    }
}

mod random;
mod stream;
