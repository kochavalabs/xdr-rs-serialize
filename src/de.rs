pub use std::io::Read;

use crate::error::Error;

pub trait XDRIn<In: Read>: Sized {
    fn read_xdr(buffer: &mut In) -> Result<Self, Error>;
}

impl<In: Read> XDRIn<In> for bool {
    fn read_xdr(buffer: &mut In) -> Result<Self, Error> {
        match i32::read_xdr(buffer) {
            Ok(1) => Ok(true),
            Ok(0) => Ok(false),
            _ => Err(Error::BoolBadFormat),
        }
    }
}

impl<In: Read> XDRIn<In> for i32 {
    fn read_xdr(buffer: &mut In) -> Result<Self, Error> {
        let mut i_bytes = [0; 4];
        match buffer.read_exact(&mut i_bytes) {
            Ok(_) => Ok(i32::from_be_bytes(i_bytes)),
            _ => Err(Error::IntegerBadFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_true() {
        let to_des: Vec<u8> = vec![0, 0, 0, 1];
        assert_eq!(true, bool::read_xdr(&mut &to_des[..]).unwrap());
    }

    #[test]
    fn test_bool_false() {
        let to_des: Vec<u8> = vec![0, 0, 0, 0];
        assert_eq!(false, bool::read_xdr(&mut &to_des[..]).unwrap());
    }

    #[test]
    fn test_bool_error() {
        let err_1: Vec<u8> = vec![0, 0, 0, 2];
        let err_2: Vec<u8> = vec![0, 0, 1, 0];
        let err_3: Vec<u8> = vec![0, 0, 0];
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut &err_1[..]));
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut &err_2[..]));
        assert_eq!(Err(Error::BoolBadFormat), bool::read_xdr(&mut &err_3[..]));
    }

    #[test]
    fn test_int() {
        let to_des: Vec<u8> = vec![255, 255, 255, 255];
        assert_eq!(-1, i32::read_xdr(&mut &to_des[..]).unwrap());
    }

    #[test]
    fn test_int_error() {
        let to_des: Vec<u8> = vec![255, 255, 255];
        assert_eq!(Err(Error::IntegerBadFormat), i32::read_xdr(&mut &to_des[..]));
    }

}
