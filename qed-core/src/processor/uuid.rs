//! UUID generation processor.
//!
//! `qed:uuid()` generates a UUID string. Supports versions 4 (random),
//! 5 (name-based SHA-1), and 7 (time-ordered random). The input is
//! ignored entirely — output is produced purely from parameters.

use super::{Processor, ProcessorError};

/// Which UUID version to generate.
#[derive(Debug)]
pub(crate) enum UuidVersion {
    /// Random UUID (RFC 9562 version 4).
    V4,
    /// Name-based SHA-1 UUID (RFC 9562 version 5).
    V5 { namespace: uuid::Uuid, name: String },
    /// Time-ordered random UUID (RFC 9562 version 7).
    V7,
}

/// Generates a UUID string in the requested version.
#[derive(Debug)]
pub(crate) struct UuidProcessor {
    pub(crate) version: UuidVersion,
}

impl Processor for UuidProcessor {
    fn execute(&self, _input: &str) -> Result<String, ProcessorError> {
        let id = match &self.version {
            UuidVersion::V4 => uuid::Uuid::new_v4(),
            UuidVersion::V5 { namespace, name } => uuid::Uuid::new_v5(namespace, name.as_bytes()),
            UuidVersion::V7 => uuid::Uuid::now_v7(),
        };
        let mut out = id.to_string();
        out.push('\n');
        Ok(out)
    }
}
