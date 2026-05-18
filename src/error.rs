use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, TranscoderError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TranscoderError {
    #[error("FFmpeg error occured: {0:?}")]
    FfmpegError(#[from] ffmpeg::Error),

    #[error("Cannot parse option `{part:?}`")]
    OptionParseError { part: String },

    #[error("Path `{path:?}` is not a file")]
    NotAFileError { path: PathBuf },

    #[error("Cannot find encoder `{name}`")]
    InvalidEncoderError { name: String },

    #[error("No availabel formats found")]
    NoAvailableFormatError,
}
