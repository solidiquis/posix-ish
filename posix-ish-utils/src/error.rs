use std::{
    error::Error as StdError,
    fmt::{self, Display},
};

pub type Result<T> = std::result::Result<T, Error>;

pub trait ToPosixishResult<T> {
    fn invalid_argument<S: Into<String>>(self, msg: S) -> Result<T>;
    fn io_error<S: Into<String>>(self, msg: S) -> Result<T>;
    fn internal<S: Into<String>>(self, msg: S) -> Result<T>;
}

#[derive(Debug)]
pub struct Error {
    code: Code,
    inner: Option<Box<dyn StdError>>,
    msg: String,
}

#[derive(Debug)]
pub enum Code {
    /// User provided in invalid argument
    InvalidArgument,
    /// IO error
    Io,
    /// Internal unexpected error
    Internal,
}

impl Error {
    pub fn invalid_argument<S: Into<String>>(msg: S) -> Self {
        Self {
            code: Code::InvalidArgument,
            inner: None,
            msg: msg.into(),
        }
    }

    pub fn io_error<S: Into<String>>(msg: S) -> Self {
        Self {
            code: Code::Io,
            inner: None,
            msg: msg.into(),
        }
    }
}

impl<T> From<Error> for Result<T> {
    fn from(err: Error) -> Self {
        Err(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! write_err {
            ($code:expr, $ctx:expr, $inner:expr) => {
                write!(
                    f,
                    "error: {}\n\n\
                    Caused by:\n  \
                    - {}\n  \
                    - {}
                    ",
                    $code, $ctx, $inner,
                )
            };
            ($code:expr, $ctx:expr) => {
                write!(
                    f,
                    "error: {}\n\n\
                    Caused by:\n  \
                    - {}
                    ",
                    $code, $ctx
                )
            };
        }
        match &self.inner {
            Some(err) => {
                write_err!(self.code, self.msg, err)
            }
            None => {
                write_err!(self.code, self.msg)
            }
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument => write!(f, "InvalidArgument"),
            Self::Io => write!(f, "I/O"),
            Self::Internal => write!(f, "Internal"),
        }
    }
}

impl<T, E> ToPosixishResult<T> for std::result::Result<T, E>
where
    E: std::error::Error + 'static,
{
    fn invalid_argument<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(Error {
                code: Code::InvalidArgument,
                inner: Some(Box::new(err)),
                msg: msg.into(),
            }),
        }
    }

    fn io_error<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(Error {
                code: Code::Io,
                inner: Some(Box::new(err)),
                msg: msg.into(),
            }),
        }
    }

    fn internal<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(Error {
                code: Code::Internal,
                inner: Some(Box::new(err)),
                msg: msg.into(),
            }),
        }
    }
}

impl<T> ToPosixishResult<T> for Option<T> {
    fn invalid_argument<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Some(val) => Ok(val),
            None => Err(Error {
                code: Code::InvalidArgument,
                inner: None,
                msg: msg.into(),
            }),
        }
    }

    fn io_error<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Some(val) => Ok(val),
            None => Err(Error {
                code: Code::Io,
                inner: None,
                msg: msg.into(),
            }),
        }
    }

    fn internal<S: Into<String>>(self, msg: S) -> Result<T> {
        match self {
            Some(val) => Ok(val),
            None => Err(Error {
                code: Code::Internal,
                inner: None,
                msg: msg.into(),
            }),
        }
    }
}
