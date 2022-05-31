use std::io::{ Error, ErrorKind, Result, Seek, SeekFrom, Write };
//use std::iter::IntoIterator;

use crate::WriteEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWavWriter<TWriter: Write + Seek> {
    writer: TWriter,
    header: WavHeader,
    data_start: u32,
    chunk_size_written: bool,
    samples_written: u32
}

impl<TWriter: Write + Seek> OpenWavWriter<TWriter> {
    pub fn new(mut writer: TWriter, header: WavHeader) -> Result<OpenWavWriter<TWriter>> {
        writer.write_str("data")?;
        writer.write_u32(0)?;

        let data_start = writer.stream_position()? as u32;

        Ok(OpenWavWriter {
            writer,
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

    pub fn get_random_access_float_writer(self) -> Result<RandomAccessWavReaderFloat<TWriter>> {
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
        self.bytes_per_sample() * 8
    }

    pub fn bytes_per_sample(&self) -> u16 {
        match self.header.sample_format {
            SampleFormat::Float => 4,
            SampleFormat:: Int24 => 3,
            SampleFormat::Int16 => 2,
            SampleFormat::Int8 => 1
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

impl<TWriter: Write + Seek> Drop for OpenWavWriter<TWriter> {
    fn drop(&mut self) {
        if !self.chunk_size_written {
            self.flush().unwrap();
        }
    }
}


pub trait RandomAccessWavWriter<T, TWriter: Write + Seek> {
    fn info(&self) -> &OpenWavWriter<TWriter>;
    fn write_sample(&mut self, sample: u32, channel: u16, value: T) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

pub struct RandomAccessWavReaderFloat<TWriter: Write + Seek> {
    open_wav: OpenWavWriter<TWriter>
}

impl<TWriter: Write + Seek> RandomAccessWavWriter<f32, TWriter> for RandomAccessWavReaderFloat<TWriter> {
    fn info(&self) -> &OpenWavWriter<TWriter> {
        &(self.open_wav)
    }

    fn write_sample(&mut self, sample: u32, channel: u16, value: f32) -> Result<()> {
        if channel >= self.open_wav.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        // Pad the file if needed
        if sample >= self.open_wav.samples_written {
            self.open_wav.writer.seek(SeekFrom::End(0))?;

            let padding_size = (self.open_wav.samples_written - sample + 1) * (self.open_wav.channels() * self.open_wav.bytes_per_sample()) as u32;
            let padding = vec![0u8; padding_size as usize];
            self.open_wav.writer.write(&padding)?;
            self.open_wav.samples_written = sample + 1;
        }

        let sample_in_channels = (sample * self.open_wav.channels() as u32) + channel as u32;
        let sample_in_bytes = (sample_in_channels as u64) * (self.open_wav.bytes_per_sample() as u64);
        let position = (self.open_wav.data_start as u64) + sample_in_bytes;

        self.open_wav.writer.seek(SeekFrom::Start(position as u64))?;
        
        self.open_wav.chunk_size_written = false;
        self.open_wav.writer.write_f32(value)
    }

    fn flush(&mut self) -> Result<()> {
        self.open_wav.flush()
    }
}