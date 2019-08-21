use crate::error::Error;

macro_rules! arr4 {
    ($s:ident) => {
        [$s[0], $s[1], $s[2], $s[3]]
    };
}

macro_rules! arr8 {
    ($s:ident) => {
        [$s[0], $s[1], $s[2], $s[3], $s[4], $s[5], $s[6], $s[7]]
    };
}

pub trait XDRIn: Sized {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error>;
}

fn consume_vec(buffer: &mut Vec<u8>, to_consume: usize) -> Result<Vec<u8>, ()> {
    if buffer.len() < to_consume {
        return Err(());
    }
    let result = buffer[0..to_consume].to_vec();
    buffer.drain(0..to_consume);
    Ok(result)
}

fn consume_padding(read: u64, buffer: &mut Vec<u8>) -> Result<((), u64), Error> {
    let to_remove: usize = ((4 - read % 4) % 4) as usize;
    match consume_vec(buffer, to_remove) {
        Ok(_) => Ok(((), to_remove as u64)),
        _ => Err(Error::InvalidPadding)
    }
}

impl XDRIn for () {
    fn read_xdr(_buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        Ok(((), 0))
    }
}

impl XDRIn for bool {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match i32::read_xdr(buffer) {
            Ok((1, 4)) => Ok((true, 4)),
            Ok((0, 4)) => Ok((false, 4)),
            _ => Err(Error::BoolBadFormat),
        }
    }
}

impl XDRIn for i32 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 4) {
            Ok(v) => Ok((i32::from_be_bytes(arr4!(v)), 4)),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

impl XDRIn for u32 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 4) {
            Ok(v) => Ok((u32::from_be_bytes(arr4!(v)), 4)),
            _ => Err(Error::UnsignedIntegerBadFormat),
        }
    }
}

impl XDRIn for i64 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 8) {
            Ok(v) => Ok((i64::from_be_bytes(arr8!(v)), 8)),
            _ => Err(Error::HyperBadFormat),
        }
    }
}

impl XDRIn for u64 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 8) {
            Ok(v) => Ok((u64::from_be_bytes(arr8!(v)), 8)),
            _ => Err(Error::UnsignedHyperBadFormat),
        }
    }
}

impl XDRIn for f32 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 4) {
            Ok(v) => Ok((f32::from_bits(u32::from_be_bytes(arr4!(v))), 4)),
            _ => Err(Error::FloatBadFormat),
        }
    }
}

impl XDRIn for f64 {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        match consume_vec(buffer, 8) {
            Ok(v) => Ok((f64::from_bits(u64::from_be_bytes(arr8!(v))), 8)),
            _ => Err(Error::DoubleBadFormat),
        }
    }
}

impl XDRIn for String {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        let size = u32::read_xdr(buffer)?.0;
        let len = size as usize;
        let mut read: u64 = 4;
        if buffer.len() < len {
            return Err(Error::StringBadFormat);
        }
        let result = String::from_utf8(buffer[0..len].to_vec());
        read += size as u64;
        let pad = consume_padding(read, buffer)?;
        Ok((result.unwrap(), read + pad.1))
    }
}

impl<T> XDRIn for Vec<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        let size = u32::read_xdr(buffer)?.0;
        let mut read: u64 = 4;
        let mut result = Vec::new();
        for _ in 0..size {
            let t_read = T::read_xdr(buffer)?;
            read += t_read.1;
            result.push(t_read.0);
        }
        let pad = consume_padding(read, buffer)?;
        Ok((result, read + pad.1))
    }
}

impl XDRIn for Vec<u8> {
    fn read_xdr(buffer: &mut Vec<u8>) -> Result<(Self, u64), Error> {
        let size = u32::read_xdr(buffer)?.0;
        let mut read: u64 = 4;
        let result = match consume_vec(buffer, size as usize) {
            Ok(res) => res,
            _ => return Err(Error::BadArraySize),
        };
        read += size as u64;
        let pad = consume_padding(read, buffer)?;
        Ok((result, read + pad.1))
    }
}

pub fn read_fixed_array<T: XDRIn>(size: u32, buffer: &mut Vec<u8>) -> Result<(Vec<T>, u64), Error> {
    let mut read = 0;
    let mut result = Vec::new();
    for _ in 0..size {
        let t_res = T::read_xdr(buffer)?;
        read += t_res.1;
        result.push(t_res.0);
    }
    Ok((result, read))
}

pub fn read_var_array<T: XDRIn>(size: u32, buffer: &mut Vec<u8>) -> Result<(Vec<T>, u64), Error> {
    let length = u32::read_xdr(buffer)?.0;
    if length > size {
        return Err(Error::BadArraySize);
    }
    let result = read_fixed_array(length, buffer)?;
    Ok((result.0, result.1 + 4))
}

pub fn read_var_opaque(max_size: u32, buffer: &mut Vec<u8>) -> Result<(Vec<u8>, u64), Error> {
    let size = u32::read_xdr(buffer)?.0;
    if size > max_size {
        return Err(Error::BadArraySize);
    }
    let mut read: u64 = 4;
    let result = match consume_vec(buffer, size as usize) {
        Ok(res) => res,
        _ => return Err(Error::BadArraySize),
    };
    read += size as u64;
    let pad = consume_padding(read, buffer)?;
    Ok((result, read + pad.1))
}

pub fn read_fixed_opaque(size: u32, buffer: &mut Vec<u8>) -> Result<(Vec<u8>, u64), Error> {
    let mut read: u64 = 0;
    let result = match consume_vec(buffer, size as usize) {
        Ok(res) => res,
        _ => return Err(Error::ByteBadFormat),
    };
    read += size as u64;
    let pad = consume_padding(read, buffer)?;
    Ok((result, read + pad.1))
}

pub fn read_var_string(max_size: u32, buffer: &mut Vec<u8>) -> Result<(String, u64), Error> {
    let size = u32::read_xdr(buffer)?.0;
    if size > max_size {
        return Err(Error::VarArrayWrongSize);
    }
    let len = size as usize;
    let mut read: u64 = 4;
    let result = String::from_utf8(buffer[0..len].to_vec());
    read += size as u64;
    let pad = consume_padding(read, buffer)?;
    Ok((result.unwrap(), read + pad.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_true() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 1];
        assert_eq!((true, 4), bool::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_bool_false() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 0];
        assert_eq!((false, 4), bool::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_bool_error() {
        let mut err_1: Vec<u8> = vec![0, 0, 0, 2];
        let mut err_2: Vec<u8> = vec![0, 0, 1, 0];
        let mut err_3: Vec<u8> = vec![0, 0, 0];
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut err_1));
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut err_2));
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut err_3));
    }

    #[test]
    fn test_int() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255];
        assert_eq!((-1, 4), i32::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_int_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(Err(Error::IntegerBadFormat), i32::read_xdr(&mut to_des));
    }

    #[test]
    fn test_uint() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255];
        assert_eq!((std::u32::MAX, 4), u32::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_uint_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(
            Err(Error::UnsignedIntegerBadFormat),
            u32::read_xdr(&mut to_des)
        );
    }

    #[test]
    fn test_hyper() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        assert_eq!((-1, 8), i64::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_hyper_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(Err(Error::HyperBadFormat), i64::read_xdr(&mut to_des));
    }

    #[test]
    fn test_uhyper() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        assert_eq!((std::u64::MAX, 8), u64::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_uhyper_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(
            Err(Error::UnsignedHyperBadFormat),
            u64::read_xdr(&mut to_des)
        );
    }

    #[test]
    fn test_float() {
        let mut to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0];
        assert_eq!((1.0, 4), f32::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_float_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(Err(Error::FloatBadFormat), f32::read_xdr(&mut to_des));
    }

    #[test]
    fn test_double() {
        let mut to_des: Vec<u8> = vec![0x3f, 0xf0, 0, 0, 0, 0, 0, 0];
        assert_eq!((1.0, 8), f64::read_xdr(&mut to_des).unwrap());
    }

    #[test]
    fn test_double_error() {
        let mut to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(Err(Error::DoubleBadFormat), f64::read_xdr(&mut to_des));
    }

    #[test]
    fn test_var_opaque_no_padding() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 8, 3, 3, 3, 4, 1, 2, 3, 4];
        let result: (Vec<u8>, u64) = Vec::read_xdr(&mut to_des).unwrap();
        assert_eq!((vec![3, 3, 3, 4, 1, 2, 3, 4], 12), result);
    }

    #[test]
    fn test_var_opaque_padding() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 5, 3, 3, 3, 4, 1, 0, 0, 0];
        let result: (Vec<u8>, u64) = Vec::read_xdr(&mut to_des).unwrap();
        assert_eq!((vec![3, 3, 3, 4, 1], 12), result);
    }

    #[test]
    fn test_var_array() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3];
        let result: (Vec<u32>, u64) = Vec::read_xdr(&mut to_des).unwrap();
        assert_eq!((vec![1, 3], 12), result);
    }

    #[test]
    fn test_var_array_error() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0];
        let result: Result<(Vec<u32>, u64), Error> = Vec::read_xdr(&mut to_des);
        assert_eq!(Err(Error::UnsignedIntegerBadFormat), result);
    }

    #[derive(XDRIn, PartialEq, Debug)]
    struct TestStruct {
        one: f32,
        two: u32,
    }

    #[test]
    fn test_struct() {
        let mut to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let expected = TestStruct { one: 1.0, two: 2 };
        let result: (TestStruct, u64) = TestStruct::read_xdr(&mut to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_struct_error() {
        let mut to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0, 0, 0, 0];
        let result: Result<(TestStruct, u64), Error> = TestStruct::read_xdr(&mut to_des);
        assert_eq!(Err(Error::UnsignedIntegerBadFormat), result);
    }

    #[test]
    fn test_string() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        assert_eq!(
            ("hello".to_string(), 12),
            String::read_xdr(&mut to_des).unwrap()
        );
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestStringLength {
        #[array(var = 5)]
        pub string: String,
    }

    #[test]
    fn test_string_length() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        let expected = TestStringLength {
            string: "hello".to_string(),
        };
        assert_eq!(
            (expected, 12),
            TestStringLength::read_xdr(&mut to_des).unwrap()
        );
    }

    #[test]
    fn test_string_length_error() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 7, 104, 101, 108, 108, 111, 0, 0, 0];
        assert_eq!(
            Err(Error::VarArrayWrongSize),
            TestStringLength::read_xdr(&mut to_des)
        );
    }

    #[derive(XDRIn, Debug, PartialEq)]
    enum TestEnum {
        Zero = 0,
        One = 1,
        Two = 2,
    }

    #[test]
    fn test_enum() {
        let mut to_des1: Vec<u8> = vec![0, 0, 0, 0];
        let mut to_des2: Vec<u8> = vec![0, 0, 0, 1];
        let mut to_des3: Vec<u8> = vec![0, 0, 0, 2];

        assert_eq!(
            (TestEnum::Zero, 4),
            TestEnum::read_xdr(&mut to_des1).unwrap()
        );
        assert_eq!(
            (TestEnum::One, 4),
            TestEnum::read_xdr(&mut to_des2).unwrap()
        );
        assert_eq!(
            (TestEnum::Two, 4),
            TestEnum::read_xdr(&mut to_des3).unwrap()
        );
    }

    #[test]
    fn test_enum_error() {
        let mut to_des1: Vec<u8> = vec![1, 0, 0, 0];
        let mut to_des2: Vec<u8> = vec![0, 1, 0, 1];
        let mut to_des3: Vec<u8> = vec![0, 0, 0, 3];

        assert_eq!(
            Err(Error::InvalidEnumValue),
            TestEnum::read_xdr(&mut to_des1)
        );
        assert_eq!(
            Err(Error::InvalidEnumValue),
            TestEnum::read_xdr(&mut to_des2)
        );
        assert_eq!(
            Err(Error::InvalidEnumValue),
            TestEnum::read_xdr(&mut to_des3)
        );
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedOpaqueNoPadding {
        #[array(fixed = 8)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_no_padding() {
        let mut to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3, 4];
        let expected = TestFixedOpaqueNoPadding {
            opaque: vec![3, 3, 3, 4, 1, 2, 3, 4],
        };
        let result = TestFixedOpaqueNoPadding::read_xdr(&mut to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_fixed_opaque_no_padding_error() {
        let mut to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3];
        let result = TestFixedOpaqueNoPadding::read_xdr(&mut to_des);
        assert_eq!(Err(Error::ByteBadFormat), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedOpaquePadding {
        #[array(fixed = 5)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_padding() {
        let mut to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 0, 0, 0];
        let expected = TestFixedOpaquePadding {
            opaque: vec![3, 3, 3, 4, 1],
        };
        let result = TestFixedOpaquePadding::read_xdr(&mut to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_fixed_opaque_padding_error() {
        let mut to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 0, 0];
        let result = TestFixedOpaquePadding::read_xdr(&mut to_des);
        assert_eq!(Err(Error::InvalidPadding), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedArray {
        #[array(fixed = 3)]
        pub data: Vec<u32>,
    }

    #[test]
    fn test_fixed_array() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3];
        let result = TestFixedArray::read_xdr(&mut to_des).unwrap();
        let expected = TestFixedArray {
            data: vec![0, 1, 3],
        };
        assert_eq!((expected, 12), result);
    }

    #[test]
    fn test_fixed_array_error() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0];
        let result = TestFixedArray::read_xdr(&mut to_des);
        assert_eq!(Err(Error::UnsignedIntegerBadFormat), result);
    }

    #[test]
    fn test_void() {
        let mut to_des: Vec<u8> = vec![];
        assert_eq!(((), 0), <()>::read_xdr(&mut to_des).unwrap());
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestVarArray {
        #[array(var = 3)]
        pub data: Vec<u32>,
    }

    #[test]
    fn test_var_array_limit() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 6];
        let result = TestVarArray::read_xdr(&mut to_des).unwrap();
        let expected = TestVarArray { data: vec![4, 6] };
        assert_eq!((expected, 12), result);
    }

    #[test]
    fn test_var_too_long() {
        let mut to_des: Vec<u8> = vec![0, 0, 0, 4];
        let result = TestVarArray::read_xdr(&mut to_des);
        assert_eq!(Err(Error::BadArraySize), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    enum TestUnion {
        First(u32),
        Second(TestStruct),
        Third(()),
    }

    #[test]
    fn test_union() {
        let mut to_des_first: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 3];
        let expected_first = TestUnion::First(3);
        let actual_first = TestUnion::read_xdr(&mut to_des_first).unwrap();
        assert_eq!((expected_first, 8), actual_first);

        let mut to_des_second: Vec<u8> = vec![0, 0, 0, 1, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let expected_second = TestUnion::Second(TestStruct { one: 1.0, two: 2 });
        let actual_second = TestUnion::read_xdr(&mut to_des_second).unwrap();
        assert_eq!((expected_second, 12), actual_second);
    }

    #[test]
    fn test_union_error() {
        let mut to_des_1: Vec<u8> = vec![0, 0, 0, 3, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        assert_eq!(
            Err(Error::InvalidEnumValue),
            TestUnion::read_xdr(&mut to_des_1)
        );

        let mut to_des_2: Vec<u8> = vec![0, 0, 0, 0, 0x3f, 0x80];
        assert_eq!(
            Err(Error::UnsignedIntegerBadFormat),
            TestUnion::read_xdr(&mut to_des_2)
        );
    }
}
