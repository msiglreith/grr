use __gl;

use device::Device;
use std;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Device {
    pub(crate) fn get_error(&self) -> Result<()> {
        let err = unsafe { self.0.GetError() };
        match err {
            __gl::OUT_OF_MEMORY => Err(Error::OutOfMemory),
            _ => Ok(()),
        }
    }
}
