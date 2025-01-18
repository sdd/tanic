//! Tanic Error Module

use crate::TanicMessage;
use figment::Error as FigmentError;
use miette::Result as MietteResult;
use std::io::Error as StdIoError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

/// Standard Tanic `Result`.
///
/// Should always be used internally in favour of other Result style types
pub type Result<T> = MietteResult<T, TanicError>;

/// Catch-all Tanic Error
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum TanicError {
    #[error("config parse error")]
    Figment(#[from] FigmentError),

    #[error("io error")]
    IoError(#[from] StdIoError),

    #[error("message send error")]
    MpscSendError(#[from] SendError<TanicMessage>),
}
