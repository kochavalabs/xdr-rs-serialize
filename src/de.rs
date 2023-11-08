extern crate base64;
extern crate hex;
extern crate json;

use crate::error::Error;
use std::rc::Rc;
use std::sync::Arc;

use json::JsonValue;

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

pub fn read_json_string<T: XDRIn>(json_str: String) -> Result<T, Error> {
    match json::parse(&json_str) {
        Ok(res) => T::read_json(res),
        Err(_) => Err(Error::invalid_json()),
    }
}

pub trait XDRIn: Sized {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error>;
    fn read_json(jval: json::JsonValue) -> Result<Self, Error>;
}

impl XDRIn for () {
    fn read_xdr(_buffer: &[u8]) -> Result<(Self, u64), Error> {
        Ok(((), 0))
    }
    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        if jval.is_string() && jval == "" {
            return Ok(());
        }
        Err(Error::invalid_json())
    }
}

impl XDRIn for bool {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        match i32::read_xdr(buffer) {
            Ok((1, 4)) => Ok((true, 4)),
            Ok((0, 4)) => Ok((false, 4)),
            _ => Err(Error::bool_bad_format()),
        }
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Boolean(val) => Ok(val),
            _ => Err(Error::bool_bad_format()),
        }
    }
}

impl XDRIn for i32 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 4 {
            return Err(Error::integer_bad_format());
        }
        let result = i32::from_be_bytes(arr4!(buffer));
        Ok((result, 4))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Number(val) => Ok(f64::from(val) as i32),
            _ => Err(Error::integer_bad_format()),
        }
    }
}

impl XDRIn for u32 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 4 {
            return Err(Error::unsigned_integer_bad_format());
        }
        let result = u32::from_be_bytes(arr4!(buffer));
        Ok((result, 4))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Number(val) => Ok(f64::from(val) as u32),
            _ => Err(Error::unsigned_integer_bad_format()),
        }
    }
}

impl XDRIn for i64 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 8 {
            return Err(Error::hyper_bad_format());
        }
        let result = i64::from_be_bytes(arr8!(buffer));
        Ok((result, 8))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        if jval.is_string() {
            if let Ok(i_val) = jval.to_string().parse::<i64>() {
                return Ok(i_val);
            }
        }
        Err(Error::hyper_bad_format())
    }
}

impl XDRIn for u64 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 8 {
            return Err(Error::unsigned_hyper_bad_format());
        }
        let result = u64::from_be_bytes(arr8!(buffer));
        Ok((result, 8))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        if jval.is_string() {
            if let Ok(i_val) = jval.to_string().parse::<u64>() {
                return Ok(i_val);
            }
        }
        Err(Error::unsigned_hyper_bad_format())
    }
}

impl XDRIn for f32 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 4 {
            return Err(Error::float_bad_format());
        }
        let result = f32::from_bits(u32::from_be_bytes(arr4!(buffer)));
        Ok((result, 4))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Number(val) => Ok(val.into()),
            _ => Err(Error::float_bad_format()),
        }
    }
}

impl XDRIn for f64 {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        if buffer.len() < 8 {
            return Err(Error::double_bad_format());
        }
        let result = f64::from_bits(u64::from_be_bytes(arr8!(buffer)));
        Ok((result, 8))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Number(num) => Ok(num.into()),
            _ => Err(Error::double_bad_format()),
        }
    }
}

impl XDRIn for String {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let size = u32::read_xdr(buffer)?.0;
        let len = size as usize;
        let mut read: u64 = 4;
        if buffer.len() < len {
            return Err(Error::string_bad_format());
        }
        let result = std::str::from_utf8(&buffer[4..len + 4])?;
        read += size as u64;
        Ok((result.to_string(), read + (4 - read % 4) % 4))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        if jval.is_string() {
            return Ok(jval.to_string());
        }
        Err(Error::string_bad_format())
    }
}

impl<T> XDRIn for Vec<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let size = u32::read_xdr(buffer)?.0;
        let mut read: u64 = 4;
        let mut result = Vec::new();
        for _ in 0..size {
            let t_read = T::read_xdr(&buffer[read as usize..])?;
            read += t_read.1;
            result.push(t_read.0);
        }
        Ok((result, read))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        let mut result = Vec::new();
        if jval.is_string() {
            match json::parse(&jval.to_string()) {
                Ok(res) => return Self::read_json(res),
                Err(_) => return Err(Error::invalid_json()),
            };
        }

        match jval {
            JsonValue::Array(vals) => {
                for val in vals {
                    result.push(T::read_json(val)?);
                }
            }
            JsonValue::Null => return Ok(result), // parse null array as empty
            _ => return Err(Error::invalid_json()),
        };
        Ok(result)
    }
}

impl XDRIn for Vec<u8> {
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let len = u32::read_xdr(buffer)?.0;
        let size = len as usize;
        let mut read: u64 = 4;
        let result = buffer[4..size + 4].to_vec();
        read += size as u64;
        Ok((result, read + (4 - read % 4) % 4))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        if jval.is_string() {
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, jval.to_string().as_bytes()) {
                Ok(val) => return Ok(val),
                _ => return Err(Error::invalid_json()),
            };
        }
        Err(Error::invalid_json())
    }
}

impl<T> XDRIn for Option<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let opted = u32::read_xdr(buffer)?.0;
        let mut read: u64 = 4;
        if opted == 0 {
            Ok((None, read))
        } else {
            let value_read = T::read_xdr(&buffer[read as usize..])?;
            read += value_read.1;
            let value = Some(value_read.0);
            Ok((value, read))
        }
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        match jval {
            JsonValue::Array(vals) if vals.len() <= 1 => match vals.into_iter().next() {
                Some(val) => {
                    let value = T::read_json(val)?;
                    Ok(Some(value))
                }
                None => Ok(None),
            },
            _ => Err(Error::invalid_json()),
        }
    }
}

impl<T> XDRIn for Rc<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let (value, read) = T::read_xdr(&buffer)?;
        Ok((Rc::new(value), read))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        let value = T::read_json(jval)?;
        Ok(Rc::new(value))
    }
}

impl<T> XDRIn for Arc<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let (value, read) = T::read_xdr(&buffer)?;
        Ok((Arc::new(value), read))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        let value = T::read_json(jval)?;
        Ok(Arc::new(value))
    }
}

impl<T> XDRIn for Box<T>
where
    T: XDRIn,
{
    fn read_xdr(buffer: &[u8]) -> Result<(Self, u64), Error> {
        let (value, read) = T::read_xdr(&buffer)?;
        Ok((Box::new(value), read))
    }

    fn read_json(jval: json::JsonValue) -> Result<Self, Error> {
        let value = T::read_json(jval)?;
        Ok(Box::new(value))
    }
}

pub fn read_fixed_array_json<T: XDRIn>(size: u32, jval: json::JsonValue) -> Result<Vec<T>, Error> {
    let result = Vec::read_json(jval)?;
    if result.len() as u32 != size {
        return Err(Error::bad_array_size());
    }
    Ok(result)
}

pub fn read_fixed_array<T: XDRIn>(size: u32, buffer: &[u8]) -> Result<(Vec<T>, u64), Error> {
    let mut read: u64 = 0;
    let mut result = Vec::new();
    for _ in 0..size {
        let t_res = T::read_xdr(&buffer[read as usize..])?;
        read += t_res.1;
        result.push(t_res.0);
    }
    Ok((result, read))
}

pub fn read_var_array_json<T: XDRIn>(
    max_size: u32,
    jval: json::JsonValue,
) -> Result<Vec<T>, Error> {
    let result = Vec::read_json(jval)?;
    if result.len() as u32 > max_size {
        return Err(Error::bad_array_size());
    }
    Ok(result)
}

pub fn read_var_array<T: XDRIn>(size: u32, buffer: &[u8]) -> Result<(Vec<T>, u64), Error> {
    let length = u32::read_xdr(buffer)?.0;
    if length > size {
        return Err(Error::bad_array_size());
    }
    let result = read_fixed_array(length, &buffer[4..])?;
    Ok((result.0, result.1 + 4))
}

pub fn read_var_opaque_json(max_size: u32, jval: json::JsonValue) -> Result<Vec<u8>, Error> {
    let result = Vec::read_json(jval)?;
    if result.len() as u32 > max_size {
        return Err(Error::bad_array_size());
    }
    Ok(result)
}

pub fn read_var_opaque(max_size: u32, buffer: &[u8]) -> Result<(Vec<u8>, u64), Error> {
    let length = u32::read_xdr(buffer)?.0;
    if length > max_size {
        return Err(Error::bad_array_size());
    }
    let result = read_fixed_opaque(length, &buffer[4..])?;
    Ok((result.0, result.1 + 4))
}

pub fn read_fixed_opaque_json(size: u32, jval: json::JsonValue) -> Result<Vec<u8>, Error> {
    if size <= 64 {
        if jval.is_string() {
            match hex::decode(jval.to_string().as_bytes()) {
                Ok(val) => return Ok(val),
                _ => return Err(Error::invalid_json()),
            };
        }
        Err(Error::invalid_json())
    } else {
        let result = Vec::read_json(jval)?;
        if result.len() as u32 != size {
            return Err(Error::bad_array_size());
        }
        Ok(result)
    }
}

pub fn read_fixed_opaque(size: u32, buffer: &[u8]) -> Result<(Vec<u8>, u64), Error> {
    let padded_size = (4 - size % 4) % 4 + size;
    if buffer.len() < padded_size as usize {
        return Err(Error::bad_array_size());
    }
    Ok((buffer[..size as usize].to_vec(), padded_size as u64))
}

pub fn read_var_string_json(max_size: u32, jval: json::JsonValue) -> Result<String, Error> {
    let result = String::read_json(jval)?;
    if result.len() as u32 > max_size {
        return Err(Error::bad_array_size());
    }
    Ok(result)
}

pub fn read_var_string(max_size: u32, buffer: &[u8]) -> Result<(String, u64), Error> {
    let length = u32::read_xdr(buffer)?.0;
    if length > max_size {
        return Err(Error::var_array_wrong_size());
    }
    String::read_xdr(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_true() {
        let to_des: Vec<u8> = vec![0, 0, 0, 1];
        assert_eq!((true, 4), bool::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_bool_true_json() {
        let to_des = "true".to_string();
        let result: bool = read_json_string(to_des).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn test_bool_false() {
        let to_des: Vec<u8> = vec![0, 0, 0, 0];
        assert_eq!((false, 4), bool::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_bool_false_json() {
        let to_des = "false".to_string();
        let result: bool = read_json_string(to_des).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn test_bool_error() {
        let err_1: Vec<u8> = vec![0, 0, 0, 2];
        let err_2: Vec<u8> = vec![0, 0, 1, 0];
        let err_3: Vec<u8> = vec![0, 0, 0];
        assert_eq!(Err(Error::bool_bad_format()), bool::read_xdr(&err_1));
        assert_eq!(Err(Error::bool_bad_format()), bool::read_xdr(&err_2));
        assert_eq!(Err(Error::bool_bad_format()), bool::read_xdr(&err_3));

        let to_des = "123".to_string();
        let result: Result<bool, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::bool_bad_format()), result);
    }

    #[test]
    fn test_int() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255];
        assert_eq!((-1, 4), i32::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_int_json() {
        let to_des = "-123".to_string();
        let result: i32 = read_json_string(to_des).unwrap();
        assert_eq!(-123, result);
    }

    #[test]
    fn test_int_error() {
        let to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(Err(Error::integer_bad_format()), i32::read_xdr(&to_des));

        let to_des = "true".to_string();
        let result: Result<i32, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::integer_bad_format()), result);
    }

    #[test]
    fn test_uint() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255];
        assert_eq!((std::u32::MAX, 4), u32::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_uint_json() {
        let to_des = "123".to_string();
        let result: u32 = read_json_string(to_des).unwrap();
        assert_eq!(123, result);
    }

    #[test]
    fn test_uint_error() {
        let to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(
            Err(Error::unsigned_integer_bad_format()),
            u32::read_xdr(&to_des)
        );

        let to_des = "true".to_string();
        let result: Result<u32, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);
    }

    #[test]
    fn test_hyper() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        assert_eq!((-1, 8), i64::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_hyper_json() {
        let to_des = r#""-123""#.to_string();
        let result: i64 = read_json_string(to_des).unwrap();
        assert_eq!(-123, result);
    }

    #[test]
    fn test_hyper_error() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(Err(Error::hyper_bad_format()), i64::read_xdr(&to_des));

        let to_des = "123".to_string();
        let result: Result<i64, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::hyper_bad_format()), result);
    }

    #[test]
    fn test_uhyper() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255, 255];
        assert_eq!((std::u64::MAX, 8), u64::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_uhyper_json() {
        let to_des = r#""123""#.to_string();
        let result: u64 = read_json_string(to_des).unwrap();
        assert_eq!(123, result);
    }

    #[test]
    fn test_uhyper_error() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(
            Err(Error::unsigned_hyper_bad_format()),
            u64::read_xdr(&to_des)
        );

        let to_des = "123".to_string();
        let result: Result<u64, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::unsigned_hyper_bad_format()), result);
    }

    #[test]
    fn test_float() {
        let to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0];
        assert_eq!((1.0, 4), f32::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_float_json() {
        let to_des = "123.321".to_string();
        let result: f32 = read_json_string(to_des).unwrap();
        assert_eq!(123.321, result);
    }

    #[test]
    fn test_float_error() {
        let to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(Err(Error::float_bad_format()), f32::read_xdr(&to_des));

        let to_des = "true".to_string();
        let result: Result<f32, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::float_bad_format()), result);
    }

    #[test]
    fn test_double() {
        let to_des: Vec<u8> = vec![0x3f, 0xf0, 0, 0, 0, 0, 0, 0];
        assert_eq!((1.0, 8), f64::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_double_json() {
        let to_des = "123.321".to_string();
        let result: f64 = read_json_string(to_des).unwrap();
        assert_eq!(123.321, result);
    }

    #[test]
    fn test_double_error() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255, 255, 255, 255];
        assert_eq!(Err(Error::double_bad_format()), f64::read_xdr(&to_des));

        let to_des = "true".to_string();
        let result: Result<f64, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::double_bad_format()), result);
    }

    #[test]
    fn test_var_opaque_no_padding() {
        let to_des: Vec<u8> = vec![0, 0, 0, 8, 3, 3, 3, 4, 1, 2, 3, 4];
        let result: (Vec<u8>, u64) = Vec::read_xdr(&to_des).unwrap();
        assert_eq!((vec![3, 3, 3, 4, 1, 2, 3, 4], 12), result);
    }

    #[test]
    fn test_var_padding_json() {
        let to_des = r#""AwMDBAECAwQEBQZkyA==""#.to_string();
        let result: Vec<u8> = read_json_string(to_des).unwrap();
        assert_eq!(vec![3, 3, 3, 4, 1, 2, 3, 4, 4, 5, 6, 100, 200], result);
    }

    #[test]
    fn test_var_array() {
        let to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3];
        let result: (Vec<u32>, u64) = Vec::read_xdr(&to_des).unwrap();
        assert_eq!((vec![1, 3], 12), result);
    }

    #[test]
    fn test_var_array_json() {
        let to_des = "[1, 2, 3, 4]".to_string();
        let result: Vec<u32> = read_json_string(to_des).unwrap();
        assert_eq!(vec![1, 2, 3, 4], result);
    }

    #[test]
    fn test_var_array_json_string() {
        let to_des = "\"[1, 2, 3, 4]\"".to_string();
        let result: Vec<u32> = read_json_string(to_des).unwrap();
        assert_eq!(vec![1, 2, 3, 4], result);
    }

    #[test]
    fn test_var_array_null() {
        let to_des = "null".to_string();
        let result: Vec<u32> = read_json_string(to_des).unwrap();
        assert_eq!(Vec::<u32>::new(), result);
    }

    #[test]
    fn test_var_array_error() {
        let to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0];
        let result: Result<(Vec<u32>, u64), Error> = Vec::read_xdr(&to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);

        let to_des = "[false]".to_string();
        let result: Result<Vec<u32>, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);
    }

    #[derive(XDRIn, PartialEq, Debug)]
    struct TestStruct {
        one: f32,
        two: u32,
    }

    #[test]
    fn test_struct() {
        let to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let expected = TestStruct { one: 1.0, two: 2 };
        let result: (TestStruct, u64) = TestStruct::read_xdr(&to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_struct_json() {
        let to_des = r#"{"one": 1.0, "two": 34}"#.to_string();
        let result: TestStruct = read_json_string(to_des).unwrap();
        let expected = TestStruct { one: 1.0, two: 34 };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_struct_error() {
        let to_des: Vec<u8> = vec![0x3f, 0x80, 0, 0, 0, 0, 0];
        let result: Result<(TestStruct, u64), Error> = TestStruct::read_xdr(&to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);

        let to_des = r#"{"asdf": 1.0, "two": 34}"#.to_string();
        let result: Result<TestStruct, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::invalid_json()), result);

        let to_des = r#"{"one": true, "two": 34}"#.to_string();
        let result: Result<TestStruct, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::float_bad_format()), result);
    }

    #[test]
    fn test_string() {
        let to_des: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        assert_eq!(
            ("hello".to_string(), 12),
            String::read_xdr(&to_des).unwrap()
        );
    }

    #[test]
    fn test_string_json() {
        let to_des = r#""hello""#.to_string();
        let result: String = read_json_string(to_des).unwrap();
        assert_eq!("hello".to_string(), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestStringLength {
        #[array(var = 5)]
        pub string: String,
    }

    #[test]
    fn test_string_length() {
        let to_des: Vec<u8> = vec![0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 0];
        let expected = TestStringLength {
            string: "hello".to_string(),
        };
        assert_eq!((expected, 12), TestStringLength::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_string_length_json() {
        let to_des = r#"{"string": "hello"}"#.to_string();
        let result: TestStringLength = read_json_string(to_des).unwrap();
        let expected = TestStringLength {
            string: "hello".to_string(),
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_string_length_error() {
        let to_des: Vec<u8> = vec![0, 0, 0, 7, 104, 101, 108, 108, 111, 0, 0, 0];
        assert_eq!(
            Err(Error::var_array_wrong_size()),
            TestStringLength::read_xdr(&to_des)
        );

        let to_des = r#"{"string": "helloasdfasdfasdfasdf"}"#.to_string();
        let result: Result<TestStringLength, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::bad_array_size()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    enum TestEnum {
        Zero = 0,
        One = 1,
        Two = 2,
    }

    #[test]
    fn test_enum() {
        let to_des1: Vec<u8> = vec![0, 0, 0, 0];
        let to_des2: Vec<u8> = vec![0, 0, 0, 1];
        let to_des3: Vec<u8> = vec![0, 0, 0, 2];

        assert_eq!((TestEnum::Zero, 4), TestEnum::read_xdr(&to_des1).unwrap());
        assert_eq!((TestEnum::One, 4), TestEnum::read_xdr(&to_des2).unwrap());
        assert_eq!((TestEnum::Two, 4), TestEnum::read_xdr(&to_des3).unwrap());
    }

    #[test]
    fn test_enum_json() {
        let to_des = "0".to_string();
        let result0: TestEnum = read_json_string(to_des).unwrap();
        let to_des = "1".to_string();
        let result1: TestEnum = read_json_string(to_des).unwrap();
        let to_des = "2".to_string();
        let result2: TestEnum = read_json_string(to_des).unwrap();

        assert_eq!(TestEnum::Zero, result0);
        assert_eq!(TestEnum::One, result1);
        assert_eq!(TestEnum::Two, result2);
    }

    #[test]
    fn test_enum_error() {
        let to_des1: Vec<u8> = vec![1, 0, 0, 0];
        let to_des2: Vec<u8> = vec![0, 1, 0, 1];
        let to_des3: Vec<u8> = vec![0, 0, 0, 3];

        assert_eq!(
            Err(Error::invalid_enum_value()),
            TestEnum::read_xdr(&to_des1)
        );
        assert_eq!(
            Err(Error::invalid_enum_value()),
            TestEnum::read_xdr(&to_des2)
        );
        assert_eq!(
            Err(Error::invalid_enum_value()),
            TestEnum::read_xdr(&to_des3)
        );

        let to_des = "4".to_string();
        let result: Result<TestEnum, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::invalid_enum_value()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedOpaqueNoPadding {
        #[array(fixed = 8)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_no_padding() {
        let to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3, 4];
        let expected = TestFixedOpaqueNoPadding {
            opaque: vec![3, 3, 3, 4, 1, 2, 3, 4],
        };
        let result = TestFixedOpaqueNoPadding::read_xdr(&to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_fixed_opaque_short_json() {
        let to_des = r#"{"opaque": "0000000000000000"}"#.to_string();
        let result: TestFixedOpaqueNoPadding = read_json_string(to_des).unwrap();
        let expected = TestFixedOpaqueNoPadding {
            opaque: vec![0, 0, 0, 0, 0, 0, 0, 0],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_fixed_opaque_no_padding_error() {
        let to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 2, 3];
        let result = TestFixedOpaqueNoPadding::read_xdr(&to_des);
        assert_eq!(Err(Error::bad_array_size()), result);

        let to_des = r#"{"opaque": "t000000000000000"}"#.to_string();
        let result: Result<TestFixedOpaqueNoPadding, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::invalid_json()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedOpaquePadding {
        #[array(fixed = 5)]
        pub opaque: Vec<u8>,
    }

    #[test]
    fn test_fixed_opaque_padding() {
        let to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 0, 0, 0];
        let expected = TestFixedOpaquePadding {
            opaque: vec![3, 3, 3, 4, 1],
        };
        let result = TestFixedOpaquePadding::read_xdr(&to_des).unwrap();
        assert_eq!((expected, 8), result);
    }

    #[test]
    fn test_fixed_opaque_padding_error() {
        let to_des: Vec<u8> = vec![3, 3, 3, 4, 1, 0, 0];
        let result = TestFixedOpaquePadding::read_xdr(&to_des);
        assert_eq!(Err(Error::bad_array_size()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedArray {
        #[array(fixed = 3)]
        pub data: Vec<u32>,
    }

    #[test]
    fn test_fixed_array() {
        let to_des: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3];
        let result = TestFixedArray::read_xdr(&to_des).unwrap();
        let expected = TestFixedArray {
            data: vec![0, 1, 3],
        };
        assert_eq!((expected, 12), result);
    }

    #[test]
    fn test_fixed_array_json() {
        let to_des = r#"{"data": [1, 2, 3]}"#.to_string();
        let result: TestFixedArray = read_json_string(to_des).unwrap();
        let expected = TestFixedArray {
            data: vec![1, 2, 3],
        };
        assert_eq!(expected, result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedArrayEmpty {
        #[array(fixed = 0)]
        pub data: Vec<u32>,
    }

    #[test]
    fn test_fixed_array_empty() {
        let to_des: Vec<u8> = vec![0, 0, 0, 0];
        let result = TestFixedArrayEmpty::read_xdr(&to_des).unwrap();
        let expected = TestFixedArrayEmpty { data: vec![] };
        assert_eq!((expected, 4), result);
    }

    #[test]
    fn test_fixed_array_empty_json() {
        let to_des = r#"{"data": []}"#.to_string();
        let result: TestFixedArrayEmpty = read_json_string(to_des).unwrap();
        let expected = TestFixedArrayEmpty { data: vec![] };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_fixed_array_empty_json_null() {
        let to_des = r#"{"data": null}"#.to_string();
        let result: TestFixedArrayEmpty = read_json_string(to_des).unwrap();
        let expected = TestFixedArrayEmpty { data: vec![] };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_fixed_array_error() {
        let to_des: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0];
        let result = TestFixedArray::read_xdr(&to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);

        let to_des = r#"{"data": [1, 2]}"#.to_string();
        let result: Result<TestFixedArray, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::bad_array_size()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestFixedArrayType {
        #[array(fixed = 3)]
        pub t: Vec<u32>,
    }

    #[test]
    fn test_fixed_array_json_type() {
        let to_des = r#"[1, 2, 3]"#.to_string();
        let result: TestFixedArrayType = read_json_string(to_des).unwrap();
        let expected = TestFixedArrayType { t: vec![1, 2, 3] };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_void() {
        let to_des: Vec<u8> = vec![];
        assert_eq!(((), 0), <()>::read_xdr(&to_des).unwrap());
    }

    #[test]
    fn test_void_json() {
        let to_des = r#""""#.to_string();
        let result: () = read_json_string(to_des).unwrap();
        assert_eq!((), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    struct TestVarArray {
        #[array(var = 3)]
        pub data: Vec<u32>,
    }

    #[test]
    fn test_var_array_limit() {
        let to_des: Vec<u8> = vec![0, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 6];
        let result = TestVarArray::read_xdr(&to_des).unwrap();
        let expected = TestVarArray { data: vec![4, 6] };
        assert_eq!((expected, 12), result);
    }

    #[test]
    fn test_var_array_json_struct() {
        let to_des = r#"{"data": [1, 2]}"#.to_string();
        let result: TestVarArray = read_json_string(to_des).unwrap();
        let expected = TestVarArray { data: vec![1, 2] };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_var_too_long() {
        let to_des: Vec<u8> = vec![0, 0, 0, 4];
        let result = TestVarArray::read_xdr(&to_des);
        assert_eq!(Err(Error::bad_array_size()), result);

        let to_des = r#"{"data": [1, 2, 3, 4]}"#.to_string();
        let result: Result<TestVarArray, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::bad_array_size()), result);
    }

    #[test]
    fn test_option() {
        let to_des_none: Vec<u8> = vec![0, 0, 0, 0];
        let result = Option::<TestEnum>::read_xdr(&to_des_none).unwrap();
        assert_eq!((None, 4), result);

        let to_des_some: Vec<u8> = vec![0, 0, 0, 1, 0, 0, 0, 2];
        let result = Option::<TestEnum>::read_xdr(&to_des_some).unwrap();
        assert_eq!((Some(TestEnum::Two), 8), result);
    }

    #[test]
    fn test_option_json() {
        let to_des_none = r#"[]"#.to_string();
        let result: Option<TestEnum> = read_json_string(to_des_none).unwrap();
        assert_eq!(None, result);

        let to_des_some = r#"[2]"#.to_string();
        let result: Option<TestEnum> = read_json_string(to_des_some).unwrap();
        assert_eq!(Some(TestEnum::Two), result);
    }

    #[test]
    fn test_option_invalid_json() {
        let to_des = r#"[2, 1]"#.to_string();
        let result: Result<Option<TestEnum>, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::invalid_json()), result);
    }

    macro_rules! test_wrap {
        ($name:ident, $name_json:ident, $typ:ident) => {
            #[test]
            fn $name() {
                let to_des: Vec<u8> = vec![0, 0, 0, 2];
                let result = $typ::<TestEnum>::read_xdr(&to_des).unwrap();
                assert_eq!(($typ::new(TestEnum::Two), 4), result);
            }

            #[test]
            fn $name_json() {
                let to_des = r#"2"#.to_string();
                let result: $typ<TestEnum> = read_json_string(to_des).unwrap();
                assert_eq!($typ::new(TestEnum::Two), result);
            }
        };
    }

    test_wrap!(test_box, test_box_json, Box);
    test_wrap!(test_rc, test_rc_json, Rc);
    test_wrap!(test_arc, test_arc_json, Arc);

    #[derive(XDRIn, Debug, PartialEq)]
    enum TestUnion {
        First(u32),
        Second(TestStruct),
        Third(()),
    }

    #[test]
    fn test_union() {
        let to_des_first: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 3];
        let expected_first = TestUnion::First(3);
        let actual_first = TestUnion::read_xdr(&to_des_first).unwrap();
        assert_eq!((expected_first, 8), actual_first);

        let to_des_second: Vec<u8> = vec![0, 0, 0, 1, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let expected_second = TestUnion::Second(TestStruct { one: 1.0, two: 2 });
        let actual_second = TestUnion::read_xdr(&to_des_second).unwrap();
        assert_eq!((expected_second, 12), actual_second);
    }

    #[test]
    fn test_union_json() {
        let to_des = r#"{"type":0,"data":3}"#.to_string();
        let result: TestUnion = read_json_string(to_des).unwrap();
        assert_eq!(TestUnion::First(3), result);

        let to_des = r#"{"type":1,"data":{"one": 1.0, "two": 2}}"#.to_string();
        let result: TestUnion = read_json_string(to_des).unwrap();
        assert_eq!(TestUnion::Second(TestStruct { one: 1.0, two: 2 }), result);

        let to_des = r#"{"type":2,"data":""}"#.to_string();
        let result: TestUnion = read_json_string(to_des).unwrap();
        assert_eq!(TestUnion::Third(()), result);
    }

    #[test]
    fn test_union_error() {
        let to_des_1: Vec<u8> = vec![0, 0, 0, 3, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        assert_eq!(
            Err(Error::invalid_enum_value()),
            TestUnion::read_xdr(&to_des_1)
        );

        let to_des_2: Vec<u8> = vec![0, 0, 0, 0, 0x3f, 0x80];
        assert_eq!(
            Err(Error::unsigned_integer_bad_format()),
            TestUnion::read_xdr(&to_des_2)
        );

        let to_des = r#"{"type":0,"data": "asdf"}"#.to_string();
        let result: Result<TestUnion, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    enum TestUnionDiscriminant {
        #[discriminant(value = "-1")]
        First(u32),
        #[discriminant(value = "1")]
        Second(TestStruct),
        #[discriminant(value = "2")]
        Third(()),
    }

    #[test]
    fn test_union_discriminant() {
        let to_des_first: Vec<u8> = vec![255, 255, 255, 255, 0, 0, 0, 3];
        let expected_first = TestUnionDiscriminant::First(3);
        let actual_first = TestUnionDiscriminant::read_xdr(&to_des_first).unwrap();
        assert_eq!((expected_first, 8), actual_first);

        let to_des_second: Vec<u8> = vec![0, 0, 0, 1, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        let expected_second = TestUnionDiscriminant::Second(TestStruct { one: 1.0, two: 2 });
        let actual_second = TestUnionDiscriminant::read_xdr(&to_des_second).unwrap();
        assert_eq!((expected_second, 12), actual_second);
    }

    #[test]
    fn test_union_discriminant_json() {
        let to_des = r#"{"type":-1,"data":3}"#.to_string();
        let result: TestUnionDiscriminant = read_json_string(to_des).unwrap();
        assert_eq!(TestUnionDiscriminant::First(3), result);

        let to_des = r#"{"type":1,"data":{"one": 1.0, "two": 2}}"#.to_string();
        let result: TestUnionDiscriminant = read_json_string(to_des).unwrap();
        assert_eq!(
            TestUnionDiscriminant::Second(TestStruct { one: 1.0, two: 2 }),
            result
        );

        let to_des = r#"{"type":2,"data":""}"#.to_string();
        let result: TestUnionDiscriminant = read_json_string(to_des).unwrap();
        assert_eq!(TestUnionDiscriminant::Third(()), result);
    }

    #[test]
    fn test_union_discriminant_error() {
        let to_des_1: Vec<u8> = vec![0, 0, 0, 0, 0x3f, 0x80, 0, 0, 0, 0, 0, 2];
        assert_eq!(
            Err(Error::invalid_enum_value()),
            TestUnionDiscriminant::read_xdr(&to_des_1)
        );

        let to_des_2: Vec<u8> = vec![255, 255, 255, 255, 0x3f, 0x80];
        assert_eq!(
            Err(Error::unsigned_integer_bad_format()),
            TestUnionDiscriminant::read_xdr(&to_des_2)
        );

        let to_des = r#"{"type":-1,"data": "asdf"}"#.to_string();
        let result: Result<TestUnionDiscriminant, Error> = read_json_string(to_des);
        assert_eq!(Err(Error::unsigned_integer_bad_format()), result);
    }

    #[derive(XDRIn, Debug, PartialEq)]
    pub struct ID {
        #[array(fixed = 32)]
        pub t: Vec<u8>,
    }

    #[derive(XDRIn, Debug, PartialEq)]
    pub struct User {
        pub id: ID,

        #[array(var = 80)]
        pub name: String,
    }

    #[test]
    fn test_array_complex() {
        let to_des = r#"[{"id":"0000000000000000000000000000000000000000000000000000000000000000","name":"sam"}]"#.to_string();
        let result: Vec<User> = read_json_string(to_des).unwrap();
        let expected: Vec<User> = vec![User {
            id: ID { t: vec![0; 32] },
            name: "sam".to_string(),
        }];
        assert_eq!(expected, result);
    }
}
