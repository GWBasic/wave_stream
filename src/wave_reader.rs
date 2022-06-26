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

    pub fn get_stream_int_8_reader(self) -> Result<StreamWavReader<i8, TReader>> {
        self.assert_int_8()?;

        Ok(StreamWavReader {
            open_wav: self,
            read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i8())
        })
    }

    pub fn get_stream_int_16_reader(self) -> Result<StreamWavReader<i16, TReader>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(StreamWavReader {
                    open_wav: self,
                    read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i16())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit unsupported"))
        }
    }

    pub fn get_stream_int_24_reader(self) -> Result<StreamWavReader<i32, TReader>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(StreamWavReader {
                    open_wav: self,
                    read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i24())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit unsupported"))
        }
    }

    pub fn get_stream_float_reader(self) -> Result<StreamWavReader<f32, TReader>> {
        self.assert_float()?;

        Ok(StreamWavReader {
            open_wav: self,
            read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_f32())
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
    pub fn get_random_access_int_8_reader(self) -> Result<RandomAccessWavReader<i8, TReader>> {
        self.assert_int_8()?;

        Ok(RandomAccessWavReader {
            open_wav: self,
            read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i8())
        })
    }

    pub fn get_random_access_int_16_reader(self) -> Result<RandomAccessWavReader<i16, TReader>> {
        match self.header.sample_format {
            SampleFormat::Int16 => {
                Ok(RandomAccessWavReader {
                    open_wav: self,
                    read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i16())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 16-bit unsupported"))
        }
    }

    pub fn get_random_access_int_24_reader(self) -> Result<RandomAccessWavReader<i32, TReader>> {
        match self.header.sample_format {
            SampleFormat::Int24 => {
                Ok(RandomAccessWavReader {
                    open_wav: self,
                    read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_i24())
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Converting to 24-bit unsupported"))
        }
    }

    pub fn get_random_access_float_reader(self) -> Result<RandomAccessWavReader<f32, TReader>> {
        self.assert_float()?;

        Ok(RandomAccessWavReader {
            open_wav: self,
            read_sample_from_stream: Box::new(|reader: &mut TReader| reader.read_f32())
        })
    }
}

pub struct RandomAccessWavReader<T, TReader: Read + Seek> {
    open_wav: OpenWavReader<TReader>,
    read_sample_from_stream: Box<dyn Fn(&mut TReader) -> Result<T>>
}

impl<T, TReader: Read + Seek> RandomAccessWavReader<T, TReader> {
    pub fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }

    pub fn read_sample(&mut self, sample: u32, channel: u16) -> Result<T> {
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

        (*self.read_sample_from_stream)(&mut self.open_wav.reader)
    }
}

pub struct StreamWavReader<T: Default + Clone, TReader: Read> {
    open_wav: OpenWavReader<TReader>,
    read_sample_from_stream: Box<dyn Fn(&mut TReader) -> Result<T>>
}

impl<T: Default + Clone, TReader: Read> StreamWavReader<T, TReader> {
    pub fn info(&self) -> &OpenWavReader<TReader> {
        &(self.open_wav)
    }
}

impl<T: Default + Clone, TReader: Read> IntoIterator for StreamWavReader<T, TReader> {
    type Item = Result<Vec<T>>;
    type IntoIter = StreamWavReaderIterator<T, TReader>;

    fn into_iter(self) -> Self::IntoIter {
        StreamWavReaderIterator {
            open_wav: self.open_wav,
            read_sample_from_stream: self.read_sample_from_stream,
            current_sample: 0
        }
    }
}

pub struct StreamWavReaderIterator<T: Default + Clone, TReader: Read> {
    open_wav: OpenWavReader<TReader>,
    read_sample_from_stream: Box<dyn Fn(&mut TReader) -> Result<T>>,
    current_sample: u32
}

impl<T: Default + Clone, TReader: Read> Iterator for StreamWavReaderIterator<T, TReader> {
    type Item = Result<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.open_wav.len_samples() {
            None
        } else {
            let num_channels: usize = self.open_wav.channels().into();
            let mut samples = vec![Default::default(); num_channels];

            for channel in 0..num_channels {
                let read_result = (*self.read_sample_from_stream)(&mut self.open_wav.reader);
                samples[channel] = match read_result {
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
