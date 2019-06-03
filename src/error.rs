#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    UnknownError,
    BoolBadFormat,
    IntegerBadFormat,
    UnsignedIntegerBadFormat,
    HyperBadFormat,
    UnsignedHyperBadFormat,
    FloatBadFormat,
    DoubleBadFormat,

    VarOpaqueBadFormat,
    FixedArrayWrongSize,
    VarArrayWrongSize,
    InvalidEnumValue,

    BadArraySize,
    InvalidPadding,
}
