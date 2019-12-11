pub use std::io::Write;

extern crate base64;
extern crate hex;

use crate::error::Error;

pub trait XDROut {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error>;
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error>;
}

fn pad(written: u64, out: &mut Vec<u8>) -> Result<u64, Error> {
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

impl XDROut for bool {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let to_write: u32 = if *self { 1 } else { 0 };
        match out.write(&to_write.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::BoolBadFormat),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let mut to_write = "true";
        if !self {
            to_write = "false";
        }
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::BoolBadFormat),
        }
    }
}

impl XDROut for i32 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::IntegerBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let to_write = self.to_string();
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDROut for u32 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::UnsignedIntegerBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let to_write = self.to_string();
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDROut for i64 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::HyperBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let to_write = format!("\"{}\"", self.to_string());
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDROut for u64 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::UnsignedHyperBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let to_write = format!("\"{}\"", self.to_string());
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDROut for f32 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_bits().to_be_bytes()) {
            Ok(4) => Ok(4),
            _ => Err(Error::FloatBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let mut to_write = self.to_string();
        if !to_write.contains(".") {
            to_write.push_str(".0")
        }
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDROut for f64 {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        match out.write(&self.to_bits().to_be_bytes()) {
            Ok(8) => Ok(8),
            _ => Err(Error::DoubleBadFormat),
        }
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let mut to_write = self.to_string();
        if !to_write.contains(".") {
            to_write.push_str(".0")
        }
        match out.write(to_write.as_bytes()) {
            Ok(len) => Ok(len as u64),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl<T> XDROut for Vec<T>
where
    T: XDROut,
{
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let mut written: u64 = 0;
        let size: u32 = self.len() as u32;
        written += size.write_xdr(out)?;
        for item in self {
            written += item.write_xdr(out)?;
        }
        Ok(written)
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        if self.len() == 0 {
            return Ok(out.write("[]".as_bytes()).unwrap() as u64);
        }

        let mut written = 0;
        written += out.write("[".as_bytes()).unwrap() as u64;
        written += self[0].write_json(out)?;
        if self.len() == 1 {
            written += out.write("]".as_bytes()).unwrap() as u64;
            return Ok(written);
        }

        for item in &self[1..] {
            written += out.write(",".as_bytes()).unwrap() as u64;
            written += item.write_json(out)?;
        }
        written += out.write("]".as_bytes()).unwrap() as u64;
        Ok(written)
    }
}

impl XDROut for Vec<u8> {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let mut written: u64 = self.len() as u64;
        let size: u32 = self.len() as u32;
        written += size.write_xdr(out)?;
        out.extend_from_slice(&self);
        written += pad(written, out)?;
        Ok(written)
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let b64 = base64::encode(&self);
        let mut written = 0;
        written += out.write("\"".as_bytes()).unwrap() as u64;

        match out.write(b64.as_bytes()) {
            Ok(len) => {
                written += len as u64;
            }
            _ => {
                return Err(Error::IntegerBadFormat);
            }
        };
        written += out.write("\"".as_bytes()).unwrap() as u64;
        Ok(written)
    }
}

impl XDROut for () {
    fn write_xdr(&self, _out: &mut Vec<u8>) -> Result<u64, Error> {
        Ok(0)
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        Ok(out.write("\"\"".as_bytes()).unwrap() as u64)
    }
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
static ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

impl XDROut for String {
    fn write_xdr(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        self.as_bytes().to_vec().write_xdr(out)
    }
    fn write_json(&self, out: &mut Vec<u8>) -> Result<u64, Error> {
        let bytes = self.as_bytes();
        let mut written = 0;
        let mut start = 0;

        written += out.write("\"".as_bytes()).unwrap();

        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }
            if start < i {
                written += out.write(&bytes[start..i]).unwrap();
            }

            let to_write = match escape {
                QU => b"\\\"",
                BS => b"\\\\",
                BB => b"\\b",
                FF => b"\\f",
                NN => b"\\n",
                RR => b"\\r",
                TT => b"\\t",
                _ => panic!("Invalid character"),
            };

            written += out.write(to_write).unwrap();

            start = i + 1
        }
        if start != bytes.len() {
            written += out.write(&bytes[start..]).unwrap();
        }
        written += out.write("\"".as_bytes()).unwrap();
        Ok(written as u64)
    }
}

pub fn write_fixed_array<T: XDROut>(val: &[T], size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 != size {
        return Err(Error::FixedArrayWrongSize);
    }
    let mut written: u64 = 0;
    for item in val {
        written += item.write_xdr(out)?;
    }
    Ok(written)
}

pub fn write_fixed_array_json<T: XDROut>(
    val: &Vec<T>,
    size: u32,
    out: &mut Vec<u8>,
) -> Result<u64, Error> {
    if val.len() as u32 != size {
        return Err(Error::FixedArrayWrongSize);
    }
    val.write_json(out)
}

pub fn write_fixed_opaque(val: &Vec<u8>, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 != size {
        return Err(Error::FixedArrayWrongSize);
    }
    out.extend(val);
    let mut written = val.len() as u64;
    written += pad(written, out)?;
    Ok(written)
}

pub fn write_fixed_opaque_json(val: &Vec<u8>, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    let len = val.len() as u32;
    if len != size {
        return Err(Error::FixedArrayWrongSize);
    }

    if len <= 64 {
        let hex = hex::encode(val);
        let mut written = 0;
        written += out.write("\"".as_bytes()).unwrap() as u64;
        match out.write(hex.as_bytes()) {
            Ok(len) => {
                written += len as u64;
            }
            _ => {
                return Err(Error::IntegerBadFormat);
            }
        };
        written += out.write("\"".as_bytes()).unwrap() as u64;
        return Ok(written);
    }
    val.write_json(out)
}

pub fn write_var_opaque(val: &Vec<u8>, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 > size {
        return Err(Error::BadArraySize);
    }
    val.write_xdr(out)
}

pub fn write_var_opaque_json(val: &Vec<u8>, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 > size {
        return Err(Error::BadArraySize);
    }
    val.write_json(out)
}

pub fn write_var_array<T: XDROut>(
    val: &Vec<T>,
    size: u32,
    out: &mut Vec<u8>,
) -> Result<u64, Error> {
    if val.len() as u32 > size {
        return Err(Error::VarArrayWrongSize);
    }
    val.write_xdr(out)
}

pub fn write_var_array_json<T: XDROut>(
    val: &Vec<T>,
    size: u32,
    out: &mut Vec<u8>,
) -> Result<u64, Error> {
    if val.len() as u32 > size {
        return Err(Error::VarArrayWrongSize);
    }
    val.write_json(out)
}

pub fn write_var_string(val: String, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 > size && size != 0 {
        return Err(Error::VarArrayWrongSize);
    }
    val.write_xdr(out)
}

pub fn write_var_string_json(val: String, size: u32, out: &mut Vec<u8>) -> Result<u64, Error> {
    if val.len() as u32 > size && size != 0 {
        return Err(Error::VarArrayWrongSize);
    }
    val.write_json(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    macro_rules! assert_json {
        ($expected:ident, $actual:ident) => {
            assert_eq!(
                str::from_utf8(&$expected).unwrap(),
                str::from_utf8(&$actual).unwrap()
            );
        };
    }

    #[test]
    fn test_bool_true() {
        let to_ser = true;
        let expected: Vec<u8> = vec![0, 0, 0, 1];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bool_true_json() {
        let to_ser = true;
        let expected: Vec<u8> = "true".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_bool_false_json() {
        let to_ser = false;
        let expected: Vec<u8> = "false".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_int_json() {
        let to_ser: i32 = -1;
        let expected: Vec<u8> = "-1".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_uint_json() {
        let to_ser: u32 = 100;
        let expected: Vec<u8> = "100".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_hyper_json() {
        let to_ser: i64 = -1;
        let expected: Vec<u8> = "\"-1\"".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_uhyper_json() {
        let to_ser: u64 = 100;
        let expected: Vec<u8> = "\"100\"".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_float_json() {
        let to_ser: f32 = 1.0;
        let expected: Vec<u8> = "1.0".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_double_json() {
        let to_ser: f64 = 1.0;
        let expected: Vec<u8> = "1.0".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_var_opaque_json() {
        let to_ser: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3, 4, 4, 5, 6, 100, 200];
        let expected: Vec<u8> = "\"AwMDBAECAwQEBQZkyA==\"".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[test]
    fn test_var_opaque_padding() {
        let to_ser: Vec<u8> = vec![3, 3, 3, 4, 1];
        let expected: Vec<u8> = vec![0, 0, 0, 5, 3, 3, 3, 4, 1, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_var_opaque_empty() {
        let to_ser: Vec<u8> = vec![];
        let expected: Vec<u8> = vec![0, 0, 0, 0];
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

    #[derive(Default, XDROut)]
    struct TestVarOpaquePadding {
        #[array(var = 5)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_var_opaque_sized_empty() {
        let to_ser = TestVarOpaquePadding { opaque: vec![] };
        let expected: Vec<u8> = vec![0, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
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
    fn test_void_json() {
        let expected: Vec<u8> = "\"\"".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        ().write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[test]
    fn test_string() {
        let to_ser: String = "hello".to_string();
        let expected: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string_json() {
        let to_ser: String = r#""hello""#.to_string();
        let expected: Vec<u8> = r#""\"hello\"""#.as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestStringLength {
        #[array(var = 5)]
        pub string: String,
    }

    #[test]
    fn test_string_length() {
        let to_ser = TestStringLength {
            string: "hello".to_string(),
        };
        let expected: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string_length_error() {
        let to_ser = TestStringLength {
            string: "hellothere".to_string(),
        };
        let mut actual: Vec<u8> = Vec::new();
        let result = to_ser.write_xdr(&mut actual);
        assert_eq!(Err(Error::VarArrayWrongSize), result);
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
        let written = to_ser.write_xdr(&mut actual).unwrap();
        assert_eq!(expected, actual);
        assert_eq!(8, written);
    }

    #[test]
    fn test_struct_json() {
        let to_ser = TestStruct { one: 1.0, two: 2 };
        let expected: Vec<u8> = r#"{"one":1.0,"two":2}"#.as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[derive(XDROut)]
    struct TestStructSingle {
        one: String,
    }

    #[test]
    fn test_struct_json_single() {
        let to_ser = TestStructSingle {
            one: "asdf".to_string(),
        };
        let expected: Vec<u8> = r#"{"one":"asdf"}"#.as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[derive(Default, XDROut)]
    struct TestFixed {
        #[array(fixed = 3)]
        pub vector: Vec<u32>,
    }

    #[derive(Default, XDROut)]
    struct TestFixedSingle {
        #[array(fixed = 32)]
        pub t: Vec<u8>,
    }

    #[test]
    fn test_fixed_array_good_json_single() {
        let mut to_ser = TestFixedSingle::default();
        to_ser.t.extend(vec![0; 32]);
        let expected: Vec<u8> =
            r#""0000000000000000000000000000000000000000000000000000000000000000""#
                .as_bytes()
                .to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
    fn test_fixed_array_good_json() {
        let mut to_ser = TestFixed::default();
        to_ser.vector.extend(vec![1, 2, 3]);
        let expected: Vec<u8> = r#"{"vector":[1,2,3]}"#.as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
    }

    #[test]
    fn test_fixed_array_bad() {
        let to_ser = TestFixed::default();
        let mut actual: Vec<u8> = Vec::new();
        let result = to_ser.write_xdr(&mut actual);
        assert_eq!(Err(Error::FixedArrayWrongSize), result);
        let result2 = to_ser.write_json(&mut actual);
        assert_eq!(Err(Error::FixedArrayWrongSize), result2);
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

    #[test]
    fn test_var_array_json() {
        let to_ser: Vec<f32> = vec![1., 2., 4.1234];
        let expected: Vec<u8> = "[1.0,2.0,4.1234]".as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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
        let result2 = to_ser.write_json(&mut actual);
        assert_eq!(Err(Error::VarArrayWrongSize), result2);
    }

    #[test]
    fn test_var_array_underflow_json() {
        let mut to_ser = TestVarOverflow::default();
        to_ser.vector.extend(vec![1, 2]);
        let expected: Vec<u8> = r#"{"vector":[1,2]}"#.as_bytes().to_vec();
        let mut actual: Vec<u8> = Vec::new();
        to_ser.write_json(&mut actual).unwrap();
        assert_json!(expected, actual);
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

    #[derive(XDROut)]
    enum TestEnum {
        Zero = 0,
        One = 1,
        Two = 2,
    }

    #[test]
    fn test_enum() {
        let expected_zero: Vec<u8> = vec![0, 0, 0, 0];
        let mut actual_zero: Vec<u8> = Vec::new();
        TestEnum::Zero.write_xdr(&mut actual_zero).unwrap();
        assert_eq!(expected_zero, actual_zero);

        let expected_one: Vec<u8> = vec![0, 0, 0, 1];
        let mut actual_one: Vec<u8> = Vec::new();
        TestEnum::One.write_xdr(&mut actual_one).unwrap();
        assert_eq!(expected_one, actual_one);

        let expected_two: Vec<u8> = vec![0, 0, 0, 2];
        let mut actual_two: Vec<u8> = Vec::new();
        TestEnum::Two.write_xdr(&mut actual_two).unwrap();
        assert_eq!(expected_two, actual_two);
    }

    #[test]
    fn test_enum_json() {
        let expected_zero: Vec<u8> = "0".as_bytes().to_vec();
        let mut actual_zero: Vec<u8> = Vec::new();
        TestEnum::Zero.write_json(&mut actual_zero).unwrap();
        assert_json!(expected_zero, actual_zero);

        let expected_one: Vec<u8> = "1".as_bytes().to_vec();
        let mut actual_one: Vec<u8> = Vec::new();
        TestEnum::One.write_json(&mut actual_one).unwrap();
        assert_json!(expected_one, actual_one);

        let expected_two: Vec<u8> = "2".as_bytes().to_vec();
        let mut actual_two: Vec<u8> = Vec::new();
        TestEnum::Two.write_json(&mut actual_two).unwrap();
        assert_json!(expected_two, actual_two);
    }

    #[derive(XDROut)]
    enum TestEnumBad {
        Value,
    }

    #[test]
    fn test_enum_bad() {
        let mut buffer: Vec<u8> = Vec::new();
        let result = TestEnumBad::Value.write_xdr(&mut buffer);
        assert_eq!(Err(Error::InvalidEnumValue), result);
        let result2 = TestEnumBad::Value.write_json(&mut buffer);
        assert_eq!(Err(Error::InvalidEnumValue), result2);
    }

    #[derive(XDROut)]
    enum TestUnion {
        First(u32),
        Second(TestStruct),
    }

    #[test]
    fn test_union() {
        let expected_first: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 3];
        let mut actual_first: Vec<u8> = Vec::new();
        let written1 = TestUnion::First(3).write_xdr(&mut actual_first).unwrap();
        assert_eq!(expected_first, actual_first);
        assert_eq!(8, written1);

        let mut actual_second: Vec<u8> = Vec::new();
        let to_ser = TestStruct { one: 1.0, two: 2 };
        let expected_second: Vec<u8> = vec![0, 0, 0, 1, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let written2 = TestUnion::Second(to_ser)
            .write_xdr(&mut actual_second)
            .unwrap();
        assert_eq!(expected_second, actual_second);
        assert_eq!(12, written2);
    }

    #[test]
    fn test_union_json() {
        let expected_first: Vec<u8> = r#"{"enum":0,"value":3}"#.as_bytes().to_vec();
        let mut actual_first: Vec<u8> = Vec::new();
        TestUnion::First(3).write_json(&mut actual_first).unwrap();
        assert_json!(expected_first, actual_first);

        let mut actual_second: Vec<u8> = Vec::new();
        let to_ser = TestStruct { one: 1.0, two: 2 };
        let expected_second: Vec<u8> =
            r#"{"enum":1,"value":{"one":1.0,"two":2}}"#.as_bytes().to_vec();
        TestUnion::Second(to_ser)
            .write_json(&mut actual_second)
            .unwrap();
        assert_json!(expected_second, actual_second);
    }
}
