//! Tanic Error Module

use figment::Error as FigmentError;
use iceberg::Error as IcebergError;
use miette::Result as MietteResult;
use std::io::Error as StdIoError;
use thiserror::Error;

/// Standard Tanic `Result`.
///
/// Should always be used internally in favour of other Result style types
pub type Result<T> = MietteResult<T, TanicError>;

/// Catch-all Tanic Error
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum TanicError {
    #[error("Config Parse Error")]
    Figment(#[from] FigmentError),

    #[error("IO Error")]
    IoError(#[from] StdIoError),

    #[error("Iceberg Error")]
    IcebergError(#[from] IcebergError),

    #[error("Unexpected")]
    UnexpectedError(String),
}

impl TanicError {
    pub fn unexpected<T: ToString>(msg: T) -> Self {
        Self::UnexpectedError(msg.to_string())
    }
}
