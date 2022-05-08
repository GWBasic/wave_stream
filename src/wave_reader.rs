use std::io::{ Error, ErrorKind, Read, Result, Seek, SeekFrom };
use std::rc::Rc;

use crate::ReadEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWav<TReader: Read + Seek> {
    wrapped: Rc<WavReaderShared<TReader>>
}

pub struct WavReaderShared<TReader: Read + Seek> {
    reader: TReader,
    header: WavHeader,
    data_start: u32,
    data_length: u32,
}

pub trait WavInfo {
    fn sample_format(&self) -> SampleFormat;
    fn channels(&self) -> u16;
    fn sample_rate(&self) -> u32;
    fn bits_per_sample(&self) -> u16;
    fn bytes_per_sample(&self) -> u16;
    fn len_samples(&self) -> u32;
}

pub struct WavReaderFloat<TReader: Read + Seek> {
    wrapped: Rc<WavReaderShared<TReader>>
}

pub trait WavReader<T> {
    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<T>;
}

impl<TReader: Read + Seek> OpenWav<TReader> {
    pub fn new(mut reader: TReader, header: WavHeader) -> Result<OpenWav<TReader>> {
        'find_data_chunk: loop {
            let chunk_name = reader.read_str(4)?;
            
            if chunk_name.eq("data") {
                break 'find_data_chunk;
            }

            let chunk_size = reader.read_u32()?;
            reader.seek(SeekFrom::Current(chunk_size as i64))?;
        }

        let data_length = reader.read_u32()?;
        let data_start = reader.stream_position()? as u32;

        Ok(OpenWav {
            wrapped: Rc::new(WavReaderShared {
                reader,
                header,
                data_start,
                data_length
            })
        })
    }

    pub fn read_float(&self) -> Result<WavReaderFloat<TReader>> {
        if self.wrapped.header.sample_format == SampleFormat::Float {
            Ok(WavReaderFloat {
                wrapped: self.wrapped.clone()
            })
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }
}

impl<TReader: Read + Seek> WavInfo for WavReaderShared<TReader> {
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
        self.header.bits_per_sample
    }

    fn bytes_per_sample(&self) -> u16 {
        self.header.bits_per_sample / 8
    }

    fn len_samples(&self) -> u32 {
        self.data_length / (self.header.bits_per_sample / 8) as u32 / self.header.channels as u32
    }
}

impl<TReader: Read + Seek> WavInfo for OpenWav<TReader> {
    fn sample_format(&self) -> SampleFormat {
        self.wrapped.sample_format()
    }

    fn channels(&self) -> u16 {
        self.wrapped.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.wrapped.sample_rate()
    }

    fn bits_per_sample(&self) -> u16 {
        self.wrapped.bits_per_sample()
    }

    fn bytes_per_sample(&self) -> u16 {
        self.wrapped.bytes_per_sample()
    }

    fn len_samples(&self) -> u32 {
        self.wrapped.len_samples()
    }
}

impl<TReader: Read + Seek> WavInfo for WavReaderFloat<TReader> {
    fn sample_format(&self) -> SampleFormat {
        self.wrapped.sample_format()
    }

    fn channels(&self) -> u16 {
        self.wrapped.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.wrapped.sample_rate()
    }

    fn bits_per_sample(&self) -> u16 {
        self.wrapped.bits_per_sample()
    }

    fn bytes_per_sample(&self) -> u16 {
        self.wrapped.bytes_per_sample()
    }

    fn len_samples(&self) -> u32 {
        self.wrapped.len_samples()
    }
}

impl<TReader: Read + Seek> WavReader<f32> for WavReaderFloat<TReader> {
    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<f32> {
        let reader = &self.wrapped.reader;

        if sample >= self.len_samples() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Sample out of range"));
        }

        if channel >= self.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        let sample_in_channels = (sample * self.channels() as u32) + channel as u32;
        let sample_in_bytes = sample_in_channels * self.bytes_per_sample() as u32;
        let position = self.wrapped.data_start + sample_in_bytes;
        reader.seek(SeekFrom::Start(position as u64))?;
        
        reader.read_f32()
    }
}
