//! Memory buffers

use std::fmt;
use std::error::Error;
use {IndexType, Backend};


/// Error creating a buffer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CreationError {
    /// Unknown other error.
    Other,
    /// Usage mode is not supported
    UnsupportedUsage(Usage),
    // TODO: unsupported role
}

impl fmt::Display for CreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreationError::UnsupportedUsage(usage) => write!(f, "{}: {:?}", self.description(), usage),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        match *self {
            CreationError::Other => "An unknown error occurred",
            CreationError::UnsupportedUsage(_) => "Requested memory usage mode is not supported",
        }
    }
}

bitflags!(
    /// Buffer usage flags.
    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    pub flags Usage: u16 {
        ///
        const TRANSFER_SRC  = 0x1,
        ///
        const TRANSFER_DST = 0x2,
        ///
        const CONSTANT    = 0x4,
        ///
        const INDEX = 0x8,
        ///
        const INDIRECT = 0x10,
        ///
        const VERTEX = 0x20,
    }
);

bitflags!(
    /// Buffer state flags.
    #[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
    pub flags Access: u16 {
        ///
        const TRANSFER_READ          = 0x01,
        ///
        const TRANSFER_WRITE         = 0x02,
        ///
        const INDEX_BUFFER_READ      = 0x10,
        ///
        const VERTEX_BUFFER_READ     = 0x20,
        ///
        const CONSTANT_BUFFER_READ   = 0x40,
        ///
        const INDIRECT_COMMAND_READ  = 0x80,
    }
);

/// Buffer state
pub type State = Access;

/// Index buffer view for `bind_index_buffer`.
pub struct IndexBufferView<'a, B: Backend> {
    ///
    pub buffer: &'a B::Buffer,
    ///
    pub offset: u64,
    ///
    pub index_type: IndexType,
}
