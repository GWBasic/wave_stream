use std::io::{ Error, ErrorKind, Read, Result, Seek, SeekFrom };
use std::iter::IntoIterator;

use crate::ReadEx;
use crate::SampleFormat;
use crate::WavHeader;

pub struct OpenWavReader<TReader: Read> {
    reader: TReader,
    header: WavHeader,
    data_length: u32,
    data_start: u32,
}

impl<TReader: Read> OpenWavReader<TReader> {
    pub fn new(mut reader: TReader, header: WavHeader, position: u32) -> Result<OpenWavReader<TReader>> {
        let mut data_start = position;
        'find_data_chunk: loop {
            let chunk_name = reader.read_str(4)?;
            data_start += 8;
            
            if chunk_name.eq("data") {
                break 'find_data_chunk;
            }

            let chunk_size = reader.read_u32()?;
            data_start += chunk_size;
            reader.skip(chunk_size as usize)?;
        }

        let data_length = reader.read_u32()?;

        Ok(OpenWavReader {
            reader,
            header,
            data_length,
            data_start
        })
    }

    fn assert_int_8(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Int8 {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to 8-bit unsupported"))
        }
    }

    fn assert_float(&self) -> Result<()> {
        if self.header.sample_format == SampleFormat::Float {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Converting to float unsupported"))
        }
    }

    pub fn get_stream_int_8_reader(self) -> Result<StreamWavReaderInt8<TReader>> {
        self.assert_int_8()?;

        Ok(StreamWavReaderInt8 {
            open_wav: self
        })
    }

    pub fn get_stream_float_reader(self) -> Result<StreamWavReaderFloat<TReader>> {
        self.assert_float()?;

        Ok(StreamWavReaderFloat {
            open_wav: self
        })
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

    pub fn len_samples(&self) -> u32 {
        self.data_length / (self.bytes_per_sample()) as u32 / self.header.channels as u32
    }
}

impl<TReader: Read + Seek> OpenWavReader<TReader> {
    fn seek_to(&mut self, sample: u32, channel: u16) -> Result<()> {
        if sample >= self.len_samples() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Sample out of range"));
        }

        if channel >= self.channels() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Channel out of range"));
        }

        let sample_in_channels = (sample * self.channels() as u32) + channel as u32;
        let sample_in_bytes = sample_in_channels * self.bytes_per_sample() as u32;
        let position = self.data_start + sample_in_bytes;
        self.reader.seek(SeekFrom::Start(position as u64))?;

        Ok(())
    }

    pub fn get_random_access_int_8_reader(self) -> Result<RandomAccessWavReaderInt8<TReader>> {
        self.assert_int_8()?;

        Ok(RandomAccessWavReaderInt8 {
            open_wav : self
        })
    }

    pub fn get_random_access_float_reader(self) -> Result<RandomAccessWavReaderFloat<TReader>> {
        self.assert_float()?;

        Ok(RandomAccessWavReaderFloat {
            open_wav : self
        })
    }
}

pub trait RandomAccessWavReader<T, TReader: Read + Seek> {
    fn info(&self) -> &OpenWavReader<TReader>;
    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<T>;
}

pub struct RandomAccessWavReaderInt8<TReader: Read + Seek> {
    open_wav: OpenWavReader<TReader>
}

pub struct RandomAccessWavReaderFloat<TReader: Read + Seek> {
    open_wav: OpenWavReader<TReader>
}

impl<TReader: Read + Seek> RandomAccessWavReader<i8, TReader> for RandomAccessWavReaderInt8<TReader> {
    fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }

    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<i8> {
        self.open_wav.seek_to(sample, channel)?;
        self.open_wav.reader.read_i8()
    }
}

impl<TReader: Read + Seek> RandomAccessWavReader<f32, TReader> for RandomAccessWavReaderFloat<TReader> {
    fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }

    fn read_sample(&mut self, sample: u32, channel: u16) -> Result<f32> {
        self.open_wav.seek_to(sample, channel)?;
        self.open_wav.reader.read_f32()
    }
}

pub struct StreamWavReaderInt8<TReader: Read> {
    open_wav: OpenWavReader<TReader>,
}

pub struct StreamWavReaderFloat<TReader: Read> {
    open_wav: OpenWavReader<TReader>,
}

pub trait StreamWavReader<T, TReader: Read> {
    fn info(&self) -> &OpenWavReader<TReader>;
}

impl<TReader: Read> StreamWavReader<i8, TReader> for StreamWavReaderInt8<TReader> {
    fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }
}

impl<TReader: Read> StreamWavReader<f32, TReader> for StreamWavReaderFloat<TReader> {
    fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }
}
 
impl<TReader: Read> IntoIterator for StreamWavReaderInt8<TReader> {
    type Item = Result<Vec<i8>>;
    type IntoIter = StreamWavReaderInt8Iterator<TReader>;

    fn into_iter(self) -> Self::IntoIter {
        StreamWavReaderInt8Iterator {
            open_wav: self.open_wav,
            current_sample: 0
        }
    }
}
 
impl<TReader: Read> IntoIterator for StreamWavReaderFloat<TReader> {
    type Item = Result<Vec<f32>>;
    type IntoIter = StreamWavReaderFloatIterator<TReader>;

    fn into_iter(self) -> Self::IntoIter {
        StreamWavReaderFloatIterator {
            open_wav: self.open_wav,
            current_sample: 0
        }
    }
}

pub struct StreamWavReaderInt8Iterator<TReader: Read> {
    open_wav: OpenWavReader<TReader>,
    current_sample: u32,
}

pub struct StreamWavReaderFloatIterator<TReader: Read> {
    open_wav: OpenWavReader<TReader>,
    current_sample: u32,
}

impl<TReader: Read> Iterator for StreamWavReaderInt8Iterator<TReader> {
    type Item = Result<Vec<i8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.open_wav.len_samples() {
            None
        } else {
            let num_channels: usize = self.open_wav.channels().into();
            let mut samples = vec![0i8; num_channels];

            for channel in 0..num_channels {
                samples[channel] = match self.open_wav.reader.read_i8() {
                    Ok(sample) => sample,
                    Err(err) => {
                        self.current_sample = u32::MAX;
                        return Some(Err(err))
                    }
                }
            }

            self.current_sample += 1;

            Some(Ok(samples))
        }
    }
}

impl<TReader: Read> Iterator for StreamWavReaderFloatIterator<TReader> {
    type Item = Result<Vec<f32>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.open_wav.len_samples() {
            None
        } else {
            let num_channels: usize = self.open_wav.channels().into();
            let mut samples = vec![0f32; num_channels];

            for channel in 0..num_channels {
                samples[channel] = match self.open_wav.reader.read_f32() {
                    Ok(sample) => sample,
                    Err(err) => {
                        self.current_sample = u32::MAX;
                        return Some(Err(err))
                    }
                }
            }

            self.current_sample += 1;

            Some(Ok(samples))
        }
    }
}