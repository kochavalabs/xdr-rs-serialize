pub mod de;
pub mod error;
pub mod ser;

#[cfg(test)]
#[macro_use]
extern crate ex_dee_derive;

pub use de::XDRIn;
pub use error::Error;
pub use ser::XDROut;
