pub mod error;

#[cfg(feature = "async")]
mod nio;
#[cfg(feature = "async")]
pub use nio::*;

#[cfg(feature = "blocking")]
pub mod blocking;
