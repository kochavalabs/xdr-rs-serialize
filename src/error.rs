#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    UnknownError,
    BoolBadFormat,
    IntegerBadFormat,
    UnsignedIntegerBadFormat,
    HyperBadFormat,
    UnsignedHyperBadFormat,
    VarOpaqueBadFormat,
    FixedArrayWrongSize,
    VarArrayWrongSize,
    InvalidEnumValue,

    BadArraySize,
    InvalidPadding,
}
