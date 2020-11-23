//! Decompression support for Tor directory connections.
//!
//! There are different compression algorithms that can be used on the
//! Tor network; right now only zlib and identity decompression are
//! supported here.
//!
//! This provides a single streaming API for decompression; we may
//! want others in the future.

use anyhow::Result;

/// Possible return conditions from a decompression operation.
#[derive(Debug, Clone)]
pub(crate) enum StatusKind {
    /// Some data was written.
    Written,
    /// We're out of space in the output buffer.
    OutOfSpace,
    /// We finished writing.
    Done,
}

/// Return value from [`Decompressor::process`].  It describes how much data
/// was transferred, and what the caller needs to do next.
#[derive(Debug, Clone)]
pub(crate) struct Status {
    /// The (successful) result of the decompression
    pub status: StatusKind,
    /// How many bytes were consumed from `inp`.
    pub consumed: usize,
    /// How many bytes were written into `out`.
    pub written: usize,
}

/// An implementation of a compression algorithm, including its state.
pub(crate) trait Decompressor {
    /// Decompress data from 'inp' into 'out'.  If 'finished' is true, no
    /// more data will be provided after the current contents of inputs.
    fn process(&mut self, inp: &[u8], out: &mut [u8], finished: bool) -> Result<Status>;
}

/// Implementation for the identity decompressor.
///
/// This does more copying than Rust best practices would prefer, but
/// we should never actually use it in practice.
pub(crate) mod identity {
    use super::{Decompressor, Status, StatusKind};
    use anyhow::Result;

    /// An identity decompressor
    pub struct Identity;

    impl Decompressor for Identity {
        fn process(&mut self, inp: &[u8], out: &mut [u8], finished: bool) -> Result<Status> {
            if out.is_empty() && !inp.is_empty() {
                return Ok(Status {
                    status: StatusKind::OutOfSpace,
                    consumed: 0,
                    written: 0,
                });
            }
            let to_copy = std::cmp::min(inp.len(), out.len());
            (&mut out[..to_copy]).copy_from_slice(&inp[..to_copy]);

            let statuskind = if finished && to_copy == inp.len() {
                StatusKind::Done
            } else {
                StatusKind::Written
            };
            Ok(Status {
                status: statuskind,
                consumed: to_copy,
                written: to_copy,
            })
        }
    }
}

/// Implementation for the [`Decompressor`] trait on [`miniz_oxide::InflateState`].
///
/// This implements zlib compression as used in Tor.
mod miniz_oxide {
    use super::{Decompressor, Status, StatusKind};

    use anyhow::{anyhow, Result};
    use miniz_oxide::inflate::stream::InflateState;
    use miniz_oxide::{MZError, MZFlush, MZStatus};

    impl Decompressor for InflateState {
        fn process(&mut self, inp: &[u8], out: &mut [u8], finished: bool) -> Result<Status> {
            let flush = if finished {
                MZFlush::Finish
            } else {
                MZFlush::None
            };
            let res = miniz_oxide::inflate::stream::inflate(self, inp, out, flush);

            let statuskind = match res.status {
                Ok(MZStatus::StreamEnd) => StatusKind::Done,
                Ok(MZStatus::Ok) => StatusKind::Written,
                Err(MZError::Buf) => StatusKind::OutOfSpace,
                other => return Err(anyhow!("compression error: {:?}", other)),
            };

            Ok(Status {
                status: statuskind,
                consumed: res.bytes_consumed,
                written: res.bytes_written,
            })
        }
    }
}