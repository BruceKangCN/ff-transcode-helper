use ffmpeg_next::Dictionary;

pub mod convert;
pub mod error;
pub mod transcode;

pub use convert::Converter;
pub use error::{Result, TranscoderError};
pub use transcode::{AudioTranscoder, VideoTranscoder};

pub fn parse_opts(opts: &str) -> Result<Dictionary<'_>> {
    let mut dict = Dictionary::new();

    for part in opts.split(',').map(str::trim) {
        let (k, v) = part
            .split_once('=')
            .ok_or(TranscoderError::OptionParseError {
                part: part.to_owned(),
            })?;
        dict.set(k, v);
    }

    Ok(dict)
}
