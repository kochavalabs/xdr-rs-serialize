pub use std::io::{Read, Write};

#[cfg(test)]
#[macro_use]
extern crate ex_dee_derive;

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

    BadArraySize,
    InvalidPadding,
}

pub trait XDROut<Out: Write> {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error>;
}

pub trait XDRIn<In: Read>: Sized {
    fn read_xdr(buffer: &mut In) -> Result<Self, Error>;
}

fn pad<Out: Write>(written: u64, out: &mut Out) -> Result<u64, Error> {
    match (4 - written % 4) % 4 {
        0 => Ok(0),
        1 => match out.write(&[0]) {
            Ok(1) => Ok(1),
            _ => Err(Error::InvalidPadding),
        },
        2 => match out.write(&[0, 0]) {
            Ok(2) => Ok(2),
            _ => Err(Error::InvalidPadding),
        },
        3 => match out.write(&[0, 0, 0]) {
            Ok(3) => Ok(3),
            _ => Err(Error::InvalidPadding),
        },
        _ => Err(Error::InvalidPadding),
    }
}

impl<Out: Write> XDROut<Out> for bool {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        let to_write: u32 = if *self { 1 } else { 0 };
        match out.write(&to_write.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::BoolBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for i32 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for u32 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::UnsignedIntegerBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for u8 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(1) => Ok(1),
            _ => Err(Error::UnsignedIntegerBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for i64 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::HyperBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for u64 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::UnsignedHyperBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for f32 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_bits().to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::UnsignedHyperBadFormat),
        }
    }
}

impl<Out: Write> XDROut<Out> for f64 {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        match out.write(&self.to_bits().to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::UnsignedHyperBadFormat),
        }
    }
}

impl<Out: Write, T: XDROut<Out> + Sized> XDROut<Out> for Vec<T> {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        let mut written: u64 = 0;
        let size: u32 = self.len() as u32;
        written += size.write_xdr(out)?;
        for item in self {
            written += item.write_xdr(out)?;
        }
        written += pad(written, out)?;
        Ok(written)
    }
}

impl<Out: Write> XDROut<Out> for () {
    fn write_xdr(&self, _out: &mut Out) -> Result<u64, Error> {
        Ok(0)
    }
}

impl<Out: Write> XDROut<Out> for String {
    fn write_xdr(&self, out: &mut Out) -> Result<u64, Error> {
        self.as_bytes().to_vec().write_xdr(out)
    }
}

pub fn write_fixed_array<Out: Write, T: XDROut<Out>>(
    val: &Vec<T>,
    size: u32,
    out: &mut Out,
) -> Result<u64, Error> {
    if val.len() as u32 != size {
        return Err(Error::FixedArrayWrongSize);
    }
    let mut written: u64 = 0;
    for item in val {
        written += item.write_xdr(out)?;
    }
    written += pad(written, out)?;
    Ok(written)
}

pub fn write_var_array<Out: Write, T: XDROut<Out>>(
    val: &Vec<T>,
    size: u32,
    out: &mut Out,
) -> Result<u64, Error> {
    if val.len() as u32 >= size {
        return Err(Error::VarArrayWrongSize);
    }
    val.write_xdr(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_true() {
        let to_ser = true;
        let expected: Vec<u8> = vec![0, 0, 0, 1];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bool_false() {
        let to_ser = false;
        let expected: Vec<u8> = vec![0, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_int() {
        let to_ser: i32 = -1;
        let expected: Vec<u8> = vec![255, 255, 255, 255];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_uint() {
        let to_ser: u32 = std::u32::MAX;
        let expected: Vec<u8> = vec![255, 255, 255, 255];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hyper() {
        let to_ser: i64 = -1;
        let expected: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_uhyper() {
        let to_ser: u64 = std::u64::MAX;
        let expected: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_float() {
        let to_ser: f32 = 1.0;
        let expected: Vec<u8> = vec![0x3f, 0x80, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_double() {
        let to_ser: f64 = 1.0;
        let expected: Vec<u8> = vec![0x3f, 0xf0, 0, 0, 0, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_var_opaque_no_padding() {
        let to_ser: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3, 4];
        let expected: Vec<u8> = vec![0, 0, 0, 8, 3, 3, 3, 4, 1, 2, 3, 4];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_var_opaque_padding() {
        let to_ser: Vec<u8> = vec![3, 3, 3, 4, 1];
        let expected: Vec<u8> = vec![0, 0, 0, 5, 3, 3, 3, 4, 1, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestFixedOpaqueNoPadding {
        #[array(fixed = 8)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_no_padding() {
        let to_ser = TestFixedOpaqueNoPadding {
            opaque: vec![3, 3, 3, 4, 1, 2, 3, 4],
        };
        let expected: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3, 4];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestFixedOpaquePadding {
        #[array(fixed = 5)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_padding() {
        let to_ser = TestFixedOpaquePadding {
            opaque: vec![3, 3, 3, 4, 1],
        };
        let expected: Vec<u8> = vec![3, 3, 3, 4, 1, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_void() {
        let expected: Vec<u8> = vec![];
        let mut actual: Vec<u8> = Vec::new();
        ().write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string() {
        let to_ser: String = "hello".to_string();
        let expected: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[derive(XDROut)]
    struct TestStruct {
        one: f32,
        two: u32,
    }

    #[test]
    fn test_struct() {
        let to_ser = TestStruct { one: 1.0, two: 2 };
        let expected: Vec<u8> = vec![0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestFixed {
        #[array(fixed = 3)]
        pub vector: Vec<u32>,
    }

    #[test]
    fn test_fixed_array_good() {
        let mut to_ser = TestFixed::default();
        to_ser.vector.extend(vec![1, 2, 3]);
        let expected: Vec<u8> = vec![0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_fixed_array_bad() {
        let to_ser = TestFixed::default();
        let mut actual: Vec<u8> = Vec::new();
        let result = to_ser.write_xdr(&mut actual);
        assert_eq!(Err(Error::FixedArrayWrongSize), result);
    }

    #[test]
    fn test_var_array() {
        let to_ser = vec![
            TestStruct { one: 1.0, two: 2 },
            TestStruct { one: 1.0, two: 3 },
        ];
        let expected: Vec<u8> = vec![
            0, 0, 0, 2, 0x3f, 0x80, 0, 0, 0, 0, 0, 2, 0x3f, 0x80, 0, 0, 0, 0, 0, 3,
        ];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestVarOverflow {
        #[array(var = 3)]
        pub vector: Vec<u32>,
    }

    #[test]
    fn test_var_array_overflow() {
        let mut to_ser = TestVarOverflow::default();
        to_ser.vector.extend(vec![1, 2, 3, 4]);
        let mut actual: Vec<u8> = Vec::new();
        let result = to_ser.write_xdr(&mut actual);
        assert_eq!(Err(Error::VarArrayWrongSize), result);
    }

    #[test]
    fn test_var_array_underflow() {
        let mut to_ser = TestVarOverflow::default();
        to_ser.vector.extend(vec![1, 2]);
        let mut actual: Vec<u8> = Vec::new();
        let expected: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2];
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }
}
