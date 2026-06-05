//! The [`Transport`] abstraction: anything that can swallow a raw HID feature
//! report.
//!
//! Decoupling the byte sink from the protocol keeps [`crate::protocol`] and
//! [`crate::device`] fully testable without hardware (via [`MockTransport`]),
//! and lets the real [`crate::hid::HidTransport`] live behind a feature flag.

use crate::error::TransportError;

/// A sink for raw HID feature reports.
pub trait Transport {
    /// Send one feature report. `data[0]` is the report id.
    fn send_feature_report(&mut self, data: &[u8]) -> Result<(), TransportError>;
}

/// A hardware-free [`Transport`] that records every report it's handed.
///
/// Useful for unit tests ("did the device layer emit the bytes I expect?") and
/// for `--dry-run`-style tooling.
#[derive(Debug, Default, Clone)]
pub struct MockTransport {
    /// Every report sent, oldest first.
    pub sent: Vec<Vec<u8>>,
}

impl MockTransport {
    /// A fresh recorder with no captured reports.
    pub fn new() -> Self {
        Self::default()
    }

    /// The most recently sent report, if any.
    pub fn last(&self) -> Option<&[u8]> {
        self.sent.last().map(Vec::as_slice)
    }

    /// How many reports have been sent.
    pub fn count(&self) -> usize {
        self.sent.len()
    }
}

impl Transport for MockTransport {
    fn send_feature_report(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.sent.push(data.to_vec());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_records_in_order() {
        let mut t = MockTransport::new();
        t.send_feature_report(&[1, 2, 3]).unwrap();
        t.send_feature_report(&[4, 5, 6]).unwrap();
        assert_eq!(t.count(), 2);
        assert_eq!(t.last(), Some(&[4, 5, 6][..]));
        assert_eq!(t.sent[0], vec![1, 2, 3]);
    }
}
