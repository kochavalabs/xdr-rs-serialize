pub use std::io::Read;

use crate::error::Error;

pub trait XDRIn<In: Read>: Sized {
    fn read_xdr(buffer: &mut In) -> Result<Self, Error>;
}
