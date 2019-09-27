#[macro_use]
extern crate xdr_rs_serialize_derive;

#[allow(unused_imports)]
use xdr_rs_serialize::de::{
    read_fixed_array, read_fixed_opaque, read_var_array, read_var_opaque, read_var_string, XDRIn,
};
#[allow(unused_imports)]
use xdr_rs_serialize::ser::{
    write_fixed_array, write_fixed_opaque, write_var_array, write_var_opaque, write_var_string,
    XDROut,
};

use xdr_rs_serialize::error::Error;

// Simple xdr struct made up of a hyper and a u_hyper.
#[derive(Default, Debug, XDROut, XDRIn)]
pub struct Simple {
    pub hyper_: i64,

    pub u_hyper_: u64,
}

// Enum example, explicit discrimination is recommended.
#[derive(Debug, XDROut, XDRIn)]
pub enum SampleEnum {
    ONE = 0,
    TWO = 1,
    Three = 2,
}

impl Default for SampleEnum {
    fn default() -> Self {
        SampleEnum::ONE
    }
}

// Signature typedef as opaque[64]
#[derive(Default, Debug, XDROut, XDRIn)]
pub struct Signature {
    #[array(fixed = 64)]
    pub t: Vec<u8>,
}

// Union of 3 possible types, Null, 32 bit integer, or the Simple struct
// defined above.
#[derive(Debug, XDROut, XDRIn)]
pub enum SampleUnion {
    NONE(()),

    T1(i32),

    T2(Simple),
}

impl Default for SampleUnion {
    fn default() -> Self {
        SampleUnion::NONE(())
    }
}

// Complex struct containing examples of most available types.
#[derive(Default, Debug, XDROut, XDRIn)]
pub struct Complex {
    // Union defined above.
    pub uni: SampleUnion,

    // Enum defined above.
    pub enu: SampleEnum,

    // A typdef of a signature.
    pub sig: Signature,

    // The boolean data type.
    pub t_bool: bool,

    // The integer data type.
    pub t_int: i32,

    // The unsigned integer data type.
    pub u_int: u32,

    // The hyper data type.
    pub hyp: i64,

    // The unsigned hyper data type.
    pub u_hyper: u64,

    // A fixed opaque data type of size 12
    #[array(fixed = 12)]
    pub fixed_opaque: Vec<u8>,

    // A variable length opaque array with max size of 122
    #[array(var = 122)]
    pub var_opaque: Vec<u8>,

    // An array of Simple structs with a fixed size of 12.
    #[array(fixed = 12)]
    pub fixed_xdr: Vec<i32>,

    // A variable array of integers with a max size of 12
    #[array(var = 12)]
    pub var_xdr: Vec<i32>,

    // A variable array string type with no size bound (2^32 chars implicit)
    pub test: String,

    // A struct
    pub sub_struct: Simple,
}

fn main() -> Result<(), Error> {
    // Create an XDR object and update it.
    let mut to_xdr = Complex::default();
    to_xdr.test = "This is a string.".to_string();

    // Default does not currently initialize arrays to the correct size,
    // so fixed arrays must be manually initialized.
    to_xdr.fixed_opaque = vec![3; 12];
    to_xdr.fixed_xdr = vec![3; 12];
    // Typedefs are wrapped in structs with the datatype at property t
    to_xdr.sig.t = vec![1; 64];

    // Write the xdr encoded bytes to a buffer.
    let mut buffer_bytes = Vec::new();
    to_xdr.write_xdr(&mut buffer_bytes)?;

    // Read the xdr bytes back to a Complex object.
    let from_xdr = Complex::read_xdr(&mut buffer_bytes.to_vec())?;
    println!("{:?}", from_xdr);

    Ok(())
}
