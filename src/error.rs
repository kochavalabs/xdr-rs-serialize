#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    UnknownError,

    Unimplemented,

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

    InvalidJson,
}
