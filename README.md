# XDR RS Serialize

[![CircleCI](https://circleci.com/gh/kochavalabs/xdr-rs-serialize.svg?style=svg)](https://circleci.com/gh/kochavalabs/xdr-rs-serialize)

Xdr-rs-serialize is a library for facilitating the (de)serialization of rust
objects into the [XDR](https://en.wikipedia.org/wiki/External_Data_Representation)
format.

## Installation

This library can be added to your project by using cargo to install the
xdr-rs-serialize crate.

```bash
cargo add xdr-rs-serialize
```

## Usage

```rust
use xdr_rs_serialize::de::XDRIn;
use xdr_rs_serialize::error::Error;
use xdr_rs_serialize::ser::XDROut;

fn main() -> Result<(), Error> {
    let mut byte_buffer = Vec::new();
    "Hello world!".to_string().write_xdr(&mut byte_buffer)?;
    // Notice that a tuple is returned with the String result at index 0 and
    // total bytes read at index 1.
    let hello_world: String = String::read_xdr(&mut &byte_buffer)?.0;
    println!("{}", hello_world);
    Ok(())
}
```

For a more complex example see the code under [example/](https://github.com/kochavalabs/xdr-rs-serialize/tree/develop/example)

## License

[MIT](https://choosealicense.com/licenses/mit/)

## Notes

- The XDR Quad type is currently not supported
