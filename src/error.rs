#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    UnknownError,

    ByteBadFormat,
    BoolBadFormat,
    IntegerBadFormat,
    UnsignedIntegerBadFormat,
    HyperBadFormat,
    UnsignedHyperBadFormat,
    FloatBadFormat,
    DoubleBadFormat,
    StringBadFormat,

    VarOpaqueBadFormat,
    FixedArrayWrongSize,
    VarArrayWrongSize,
    InvalidEnumValue,

    BadArraySize,
    InvalidPadding,
}
