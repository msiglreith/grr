
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NoError,
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    InvalidFramebufferOperation,
    OutOfMemory,
    StackUnderflow,
    StackOverflow,

    Unknown,
}

impl From<GLenum> for Error {
    fn from(err: GLenum) -> Self {
        match err {
            __gl::NO_ERROR => Error::NoError,
            __gl::INVALID_ENUM => Error::InvalidEnum,
            __gl::INVALID_VALUE => Error::InvalidValue,
            __gl::INVALID_OPERATION => Error::InvalidOperation,
            __gl::INVALID_FRAMEBUFFER_OPERATION => Error::InvalidFramebufferOperation,
            __gl::OUT_OF_MEMORY => Error::OutOfMemory,
            __gl::STACK_UNDERFLOW => Error::StackUnderflow,
            __gl::STACK_OVERFLOW => Error::StackOverflow,
            _ => Error::Unknown,
        }
    }
}
