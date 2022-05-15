use std::io::{ Error, ErrorKind, Read, Result, Seek, SeekFrom };

use crate::ReadEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWav<TReader: Read + Seek> {
    reader: TReader,
    header: WavHeader,
    data_start: u32,
    data_length: u32,
}

pub struct RandomAccessWavReaderFloat<TReader: Read + Seek> {
    open_wav: OpenWav<TReader>
}

pub trait RandomAccessWavReader<T, TReader: Read + Seek> {
    fn info(&self) -> &OpenWav<TReader>;
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
            reader,
            header,
            data_start,
            data_length
        })
    }

    pub fn as_random_access_float(self) -> Result<RandomAccessWavReaderFloat<TReader>> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(RandomAccessWavReaderFloat {
                open_wav: self
            })
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }

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

    pub fn len_samples(&self) -> u32 {
        self.data_length / (self.header.bits_per_sample / 8) as u32 / self.header.channels as u32
    }
}

impl<TReader: Read + Seek> RandomAccessWavReader<f32, TReader> for RandomAccessWavReaderFloat<TReader> {
    fn info(&self) -> &OpenWav<TReader> {
        &(self.open_wav)
    }

    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<f32> {
        if sample >= self.open_wav.len_samples() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Sample out of range"));
        }

        if channel >= self.open_wav.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        let sample_in_channels = (sample * self.open_wav.channels() as u32) + channel as u32;
        let sample_in_bytes = sample_in_channels * self.open_wav.bytes_per_sample() as u32;
        let position = self.open_wav.data_start + sample_in_bytes;
        self.open_wav.reader.seek(SeekFrom::Start(position as u64))?;
        
        self.open_wav.reader.read_f32()
    }
}
