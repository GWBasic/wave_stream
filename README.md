# wave_stream
A streaming wav reader and writer for Rust. Wave_stream supports the following:
- Reading, both in random access and streaming modes
- Writing in random access mode.

Wave_stream supports any sample rate. It supports 8-bit, 16-bit, 24-bit, and floating-point wave files.

Wave_stream does not load the entire wav file into RAM. This allows working with extremely large files with low RAM overhead.

For a full example, and instructions, see: https://github.com/GWBasic/wave_stream_example/blob/main/src/main.rs

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Note: Pull requests require that "cargo fmt" is run. If you are using Visual Studio Code, enable "Format on Save": https://stackoverflow.com/a/67861602/1711103