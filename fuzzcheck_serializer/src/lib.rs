//! This crate contains types implementing the Serializer trait of fuzzcheck.
//! There are currently two implementations:
//!
//! * SerdeSerializer uses the `serde` and `serde_json` crate to serialize
//! the test inputs (of arbitrary Serializable type) to a `.json` file.
//! It is available under the “serde” feature
//!
//! * [ByteSerializer] encodes and decodes values of type `Vec<u8>` by simply
//! copy/pasting the bytes from/to the files. The extension is customizable.
//!

#![feature(no_coverage)]

#[cfg(feature = "serde-json")]
mod serde_serializer;
#[cfg(feature = "serde-json")]
pub use serde_serializer::SerdeSerializer;

#[cfg(feature = "serde-json-alternative")]
mod json_serializer;
#[cfg(feature = "serde-json-alternative")]
pub use decent_serde_json_alternative;
#[cfg(feature = "serde-json-alternative")]
pub use json;
#[cfg(feature = "serde-json-alternative")]
pub use json_serializer::JsonSerializer;

/**
A Serializer for Vec<u8> that simply copies the bytes from/to the files.

```ignore
#[no_coverage] fn parse(data: &[u8]) -> bool {
    // ...
}
let mutator = VecMutator<U8Mutator>::default();

// pass the desired extension of the file as argument
let serializer = ByteSerializer::new("png");

fuzzcheck::launch(parse, mutator, serializer)
```
*/
pub struct ByteSerializer {
    ext: &'static str,
}

impl ByteSerializer {
    #[no_coverage]
    pub fn new(ext: &'static str) -> Self {
        Self { ext }
    }
}

impl fuzzcheck_traits::Serializer for ByteSerializer {
    type Value = Vec<u8>;
    #[no_coverage]
    fn is_utf8(&self) -> bool {
        false
    }
    #[no_coverage]
    fn extension(&self) -> &str {
        self.ext
    }
    #[no_coverage]
    fn from_data(&self, data: &[u8]) -> Option<Self::Value> {
        Some(data.into())
    }
    #[no_coverage]
    fn to_data(&self, value: &Self::Value) -> Vec<u8> {
        value.clone()
    }
}
