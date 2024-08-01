use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::Error as IOErr;
use std::string::FromUtf8Error;

use packet::errors::PacketError;
use storage::StorageError;

#[derive(Debug)]
pub enum ServerError {
    IOError(IOErr),
    FromUtf8Error(FromUtf8Error),
    StorageError(StorageError),
    InvalidData,
    PacketError(PacketError),
}

impl Error for ServerError {}

impl From<IOErr> for ServerError {
    fn from(e: IOErr) -> Self {
        ServerError::IOError(e)
    }
}

impl From<FromUtf8Error> for ServerError {
    fn from(e: FromUtf8Error) -> Self {
        ServerError::FromUtf8Error(e)
    }
}

impl From<StorageError> for ServerError {
    fn from(e: StorageError) -> Self {
        ServerError::StorageError(e)
    }
}

impl From<PacketError> for ServerError {
    fn from(e: PacketError) -> Self {
        ServerError::PacketError(e)
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "IOError - {e}")
            }
            Self::FromUtf8Error(ref msg) => {
                write!(f, "RespError - {msg}")
            }
            Self::StorageError(e) => {
                write!(f, "StorageError - {e}")
            }
            Self::InvalidData => {
                write!(f, "InvalidData")
            }
            Self::PacketError(e) => {
                write!(f, "PacketError - {e}")
            }
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;
