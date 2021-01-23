use core::fmt::{Debug, Display};

pub trait HeaplessErrorExt {
    type Ok;

    fn context(self, msg: &'static str) -> HeaplessResult<Self::Ok>;
}

#[derive(Debug, Clone)]
pub enum HeaplessError {
    StrError(&'static str),
}

impl HeaplessError {
    pub fn msg(msg: &'static str) -> Self {
        Self::StrError(msg)
    }
}

impl From<&'static str> for HeaplessError {
    fn from(err: &'static str) -> Self {
        Self::StrError(err)
    }
}

impl Display for HeaplessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StrError(s) => write!(f, "{}", s),
        }
    }
}

pub type HeaplessResult<T> = Result<T, HeaplessError>;

impl<T> HeaplessErrorExt for Option<T> {
    type Ok = T;

    fn context(self, msg: &'static str) -> HeaplessResult<Self::Ok> {
        match self {
            Some(s) => Ok(s),
            None => Err(HeaplessError::StrError(msg)),
        }
    }
}

// impl<T, E: Into<HeaplessError>> HeaplessTry for Result<T, E> {
//     type Ok = T;

//     fn fix(self) -> HeaplessResult<T> {
//         self.map_err(|e| e.into())
//     }
// }
