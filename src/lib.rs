pub mod de;
pub mod error;
pub mod ser;

#[cfg(test)]
#[macro_use]
extern crate ex_dee_derive;

pub use de::{read_fixed_array, read_var_array, read_var_string, XDRIn};
pub use error::Error;
pub use ser::{write_fixed_array, write_var_array, write_var_string, XDROut};
