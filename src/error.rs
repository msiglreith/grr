/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Error Handling

use __gl;

use device::Device;
use std::{error, fmt, result};

/// Error return codes
///
/// Error handling in `grr` only deals with runtime-only detectable errors.
///
/// Other error codes returned by OpenGL are either treated as API miss use (see `Valid Usage` sections),
/// or indicate driver or implementation issues.
///
/// API validation is provided by the debug functionality on device creation.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
}

/// A specialized Result type for `grr` operations.
pub type Result<T> = result::Result<T, Error>;

impl Device {
    pub(crate) fn get_error(&self) -> Result<()> {
        let err = unsafe { self.0.GetError() };
        match err {
            __gl::OUT_OF_MEMORY => Err(Error::OutOfMemory),
            _ => Ok(()),
        }
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::OutOfMemory => write!(fmt, "OutOfMemory"),
        }
    }
}
