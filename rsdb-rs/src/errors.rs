use std::io::Error as IOErr;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum RsDBError {
    IOError(IOErr),
    RespError(String),
    NotConnect,
    NoDbSelected,
}

impl Error for RsDBError{}

impl From<IOErr> for RsDBError {
    fn from(e: IOErr) -> Self {
        RsDBError::IOError(e)
    }
}

impl Display for RsDBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "IOError - {e}")
            }
            Self::RespError(ref msg) => {
                write!(f, "RespError - {msg}")
            }
            Self::NotConnect => {
                write!(f, "Not connect to server")
            }
            Self::NoDbSelected => {
                write!(f, "No database selected")
            }
        }
    }
}

pub type RsDBResult<T> = Result<T, RsDBError>;
