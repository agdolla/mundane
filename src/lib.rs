// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

//! Cryptography in Rust.
//!
//! Mundane is a Rust cryptography library backed by BoringSSL that is difficult
//! to misuse, ergonomic, and performant (in that order).
//!
//! # Features
//!
//! By default, Mundane provides only high-level cryptographic primitives.
//! Unless you are implementing cryptographic protocols, these high-level
//! primitives should be all you need. However, if you are sure that you need
//! something lower level, Mundane provides features to enable a number of
//! different low level primitives.
//!
//! WARNING: Being low level, these primitives provide the programmer with more
//! degrees of freedom. There are more conditions that the programmer must meet
//! in order to guarantee security, and thus more ways for the programmer to
//! shoot themself in the foot. Please only use these primitives if you're aware
//! of the risks and are comfortable with the responsibility of using them
//! correctly!
//!
//! **Features**
//!
//! | Name           | Description              |
//! | -------------- | ------------------------ |
//! | `kdf`          | Key derivation functions |
//! | `rand-bytes`   | Generate random bytes    |
//! | `rsa-pkcs1v15` | RSA-PKCS1v1.5 signatures |
//!
//! # Insecure Operations
//!
//! Mundane supports one additional feature not listed in the previous section:
//! `insecure`. This enables some cryptographic primitives which are today
//! considered insecure. These should only be used for compatibility with legacy
//! systems - never in new systems! When the `insecure` feature is used, an
//! `insecure` module is added to the crate root. All insecure primitives are
//! exposed through this module.

#![doc(html_root_url = "https://joshlf.com/rustdoc/mundane")]
#![deny(missing_docs)]
#![deny(warnings)]
// just in case we forget to add #[forbid(unsafe_code)] on new module
// definitions
#![deny(unsafe_code)]

#[cfg(test)]
extern crate lazy_static;

#[macro_use]
mod macros;

// Forbid unsafe code except in the boringssl module.
#[allow(unsafe_code)]
mod boringssl;
#[forbid(unsafe_code)]
pub mod hash;
#[forbid(unsafe_code)]
pub mod hmac;
#[cfg(feature = "insecure")]
#[forbid(unsafe_code)]
pub mod insecure;
#[cfg(feature = "kdf")]
#[forbid(unsafe_code)]
pub mod kdf;
#[forbid(unsafe_code)]
pub mod password;
#[forbid(unsafe_code)]
pub mod public;
#[forbid(unsafe_code)]
mod util;

use std::fmt::{self, Debug, Display, Formatter};

use boringssl::BoringError;

/// Reads cryptographically-secure random bytes.
///
/// This is a low-level primitive often used to construct higher-level
/// protocols. Unless you're sure that this is what you need, you should
/// probably be using something else. For example, all key types can be randomly
/// generated using higher-level functions (e.g., [`EcPrivKey::generate`]),
/// scrypt nonces are generated using the [`scrypt_generate`] function, etc.
///
/// [`EcPrivKey::generate`]: ::public::ec::EcPrivKey::generate
/// [`scrypt_generate`]: ::password::scrypt::scrypt_generate
#[cfg(feature = "rand-bytes")]
pub fn rand_bytes(bytes: &mut [u8]) {
    boringssl::rand_bytes(bytes);
}

/// Errors generated by this crate.
///
/// `Error` represents two types of errors: errors generated by BoringSSL, and
/// errors generated by the Rust code in this crate. When printed (using either
/// `Display` or `Debug`), BoringSSL errors are of the form `boringssl:
/// <error>`, while errors generated by Rust code are of the form `<error>`.
pub struct Error(ErrorInner);

impl Error {
    fn new(s: String) -> Error {
        Error(ErrorInner::Mundane(s))
    }
}

#[doc(hidden)]
impl From<BoringError> for Error {
    fn from(err: BoringError) -> Error {
        Error(ErrorInner::Boring(err))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match &self.0 {
            ErrorInner::Mundane(err) => write!(f, "{}", err),
            ErrorInner::Boring(err) => write!(f, "boringssl: {}", err),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match &self.0 {
            ErrorInner::Mundane(err) => write!(f, "{}", err),

            ErrorInner::Boring(err) => {
                if err.stack_depth() == 1 {
                    // Either there was no stack trace, or the stack trace only
                    // contained a single frame. In either case, don't bother
                    // printing a preceding newline.
                    write!(f, "boringssl: {:?}", err)
                } else {
                    // There's a multi-line stack trace, so print a preceding
                    // newline.
                    write!(f, "boringssl:\n{:?}", err)
                }
            }
        }
    }
}

impl std::error::Error for Error {}

enum ErrorInner {
    Mundane(String),
    Boring(BoringError),
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }
}
