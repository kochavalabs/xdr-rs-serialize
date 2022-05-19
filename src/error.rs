use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    BoolBadFormat,
    IntegerBadFormat,
    UnsignedIntegerBadFormat,
    HyperBadFormat,
    UnsignedHyperBadFormat,
    FloatBadFormat,
    DoubleBadFormat,
    StringBadFormat,

    FixedArrayWrongSize,
    VarArrayWrongSize,
    InvalidEnumValue,

    BadArraySize,
    InvalidPadding,

    InvalidJson,

    Utf8Error(std::str::Utf8Error),
}

#[derive(Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            _ => write!(f, "{:?}", self)
        }
    }
}

impl Error {
    fn from_kind(kind: ErrorKind) -> Self {
        Error { kind }
    }

    fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn bool_bad_format() -> Self {
        Error {
            kind: ErrorKind::BoolBadFormat,
        }
    }

    pub fn integer_bad_format() -> Self {
        Error {
            kind: ErrorKind::IntegerBadFormat,
        }
    }

    pub fn unsigned_integer_bad_format() -> Self {
        Error {
            kind: ErrorKind::UnsignedIntegerBadFormat,
        }
    }

    pub fn hyper_bad_format() -> Self {
        Error {
            kind: ErrorKind::HyperBadFormat,
        }
    }

    pub fn unsigned_hyper_bad_format() -> Self {
        Error {
            kind: ErrorKind::UnsignedHyperBadFormat,
        }
    }

    pub fn float_bad_format() -> Self {
        Error {
            kind: ErrorKind::FloatBadFormat,
        }
    }

    pub fn double_bad_format() -> Self {
        Error {
            kind: ErrorKind::DoubleBadFormat,
        }
    }

    pub fn string_bad_format() -> Self {
        Error {
            kind: ErrorKind::StringBadFormat,
        }
    }

    pub fn fixed_array_wrong_size() -> Self {
        Error {
            kind: ErrorKind::FixedArrayWrongSize,
        }
    }

    pub fn var_array_wrong_size() -> Self {
        Error {
            kind: ErrorKind::VarArrayWrongSize,
        }
    }

    pub fn invalid_enum_value() -> Self {
        Error {
            kind: ErrorKind::InvalidEnumValue,
        }
    }

    pub fn bad_array_size() -> Self {
        Error {
            kind: ErrorKind::BadArraySize,
        }
    }

    pub fn invalid_padding() -> Self {
        Error {
            kind: ErrorKind::InvalidPadding,
        }
    }

    pub fn invalid_json() -> Self {
        Error {
            kind: ErrorKind::InvalidJson,
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(utf_err: std::str::Utf8Error) -> Self {
        Error::from_kind(ErrorKind::Utf8Error(utf_err))
    }
}