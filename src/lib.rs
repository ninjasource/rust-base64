//! # Configs
//!
//! There isn't just one type of Base64; that would be too simple. You need to choose a character
//! set (standard, URL-safe, etc) and padding suffix (yes/no).
//! The `Config` struct encapsulates this info. There are some common configs included: `STANDARD`,
//! `URL_SAFE`, etc. You can also make your own `Config` if needed.
//!
//! The functions that don't have `config` in the name (e.g. `encode()` and `decode()`) use the
//! `STANDARD` config .
//!
//! The functions that write to a slice (the ones that end in `_slice`) are generally the fastest
//! because they don't need to resize anything. If it fits in your workflow and you care about
//! performance, keep using the same buffer (growing as need be) and use the `_slice` methods for
//! the best performance.
//!
//! # Encoding
//!
//! Several different encoding functions are available to you depending on your desire for
//! convenience vs performance.
//!
//! | Function                | Output                       | Allocates                      |
//! | ----------------------- | ---------------------------- | ------------------------------ |
//! | `encode`                | Returns a new `String`       | Always                         |
//! | `encode_config`         | Returns a new `String`       | Always                         |
//! | `encode_config_buf`     | Appends to provided `String` | Only if `String` needs to grow |
//! | `encode_config_slice`   | Writes to provided `&[u8]`   | Never                          |
//!
//! All of the encoding functions that take a `Config` will pad as per the config.
//!
//! # Decoding
//!
//! Just as for encoding, there are different decoding functions available.
//!
//! Note that all decode functions that take a config will allocate a copy of the input if you
//! specify a config that requires whitespace to be stripped. If you care about speed, don't use
//! formats that require whitespace stripping.
//!
//! | Function                | Output                        | Allocates                      |
//! | ----------------------- | ----------------------------- | ------------------------------ |
//! | `decode`                | Returns a new `Vec<u8>`       | Always                         |
//! | `decode_config`         | Returns a new `Vec<u8>`       | Always                         |
//! | `decode_config_buf`     | Appends to provided `Vec<u8>` | Only if `Vec` needs to grow    |
//! | `decode_config_slice`   | Writes to provided `&[u8]`    | Never                          |
//!
//! Unlike encoding, where all possible input is valid, decoding can fail (see `DecodeError`).
//!
//! Input can be invalid because it has invalid characters or invalid padding. (No padding at all is
//! valid, but excess padding is not.)
//!
//! Whitespace in the input is invalid unless `strip_whitespace` is enabled in the `Config` used.
//!
//! # Panics
//!
//! If length calculations result in overflowing `usize`, a panic will result.
//!
//! The `_slice` flavors of encode or decode will panic if the provided output slice is too small,

#![deny(
    missing_docs, trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
    unused_results, variant_size_differences, warnings
)]

extern crate byteorder;

mod chunked_encoder;
pub mod display;
mod tables;

mod encode;
pub use encode::{encode, encode_config, encode_config_buf, encode_config_slice};

mod decode;
pub use decode::{decode, decode_config, decode_config_buf, decode_config_slice, DecodeError};

#[cfg(test)]
mod tests;

/// Available encoding character sets
#[derive(Clone, Copy, Debug)]
pub enum CharacterSet {
    /// The standard character set (uses `+` and `/`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-3).
    Standard,
    /// The URL safe character set (uses `-` and `_`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-4).
    UrlSafe,
    /// The `crypt(3)` character set (uses `./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz`).
    ///
    /// Not standardized, but folk wisdom on the net asserts that this alphabet is what crypt uses.
    Crypt,
}

impl CharacterSet {
    fn encode_table(&self) -> &'static [u8; 64] {
        match *self {
            CharacterSet::Standard => tables::STANDARD_ENCODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_ENCODE,
            CharacterSet::Crypt => tables::CRYPT_ENCODE,
        }
    }

    fn decode_table(&self) -> &'static [u8; 256] {
        match *self {
            CharacterSet::Standard => tables::STANDARD_DECODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_DECODE,
            CharacterSet::Crypt => tables::CRYPT_DECODE,
        }
    }
}


/// Contains configuration parameters for base64 encoding
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Character set to use
    char_set: CharacterSet,
    /// True to pad output with `=` characters
    pad: bool,
    /// Remove whitespace before decoding, at the cost of an allocation. Whitespace is defined
    /// according to POSIX-locale `isspace`, meaning \n \r \f \t \v and space.
    strip_whitespace: bool,
}

impl Config {
    /// Create a new `Config`.
    pub fn new(
        char_set: CharacterSet,
        pad: bool,
        strip_whitespace: bool
    ) -> Config {
        Config {
            char_set,
            pad,
            strip_whitespace,
        }
    }
}

/// Standard character set with padding.
pub const STANDARD: Config = Config {
    char_set: CharacterSet::Standard,
    pad: true,
    strip_whitespace: false,
};

/// Standard character set without padding.
pub const STANDARD_NO_PAD: Config = Config {
    char_set: CharacterSet::Standard,
    pad: false,
    strip_whitespace: false,
};

/// URL-safe character set with padding
pub const URL_SAFE: Config = Config {
    char_set: CharacterSet::UrlSafe,
    pad: true,
    strip_whitespace: false,
};

/// URL-safe character set without padding
pub const URL_SAFE_NO_PAD: Config = Config {
    char_set: CharacterSet::UrlSafe,
    pad: false,
    strip_whitespace: false,
};

/// As per `crypt(3)` requirements
pub const CRYPT: Config = Config {
    char_set: CharacterSet::Crypt,
    pad: false,
    strip_whitespace: false,
};
