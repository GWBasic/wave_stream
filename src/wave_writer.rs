use std::io::{ /*Error, ErrorKind, Read, Result, Seek, SeekFrom,*/ Write };
/*use std::iter::IntoIterator;

use crate::ReadEx;
use crate::SampleFormat;*/
use crate::WavHeader;

pub struct OpenWavWriter<TWriter: Write> {
    writer: TWriter,
    header: WavHeader,
    data_start: u32,
}
