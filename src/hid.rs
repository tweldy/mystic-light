//! The real USB-HID transport, backed by [`hidapi`].
//!
//! Only compiled when the `hidapi` feature is enabled (it is, by default).

use hidapi::{HidApi, HidDevice};

use crate::error::{Error, TransportError};
use crate::protocol::{PRODUCT_ID, VENDOR_ID};
use crate::transport::Transport;

/// A [`Transport`] that writes feature reports to a Mystic Light HID device.
pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    /// Open the first connected Mystic Light device, creating a fresh
    /// [`HidApi`] context.
    pub fn open() -> Result<Self, Error> {
        let api = HidApi::new().map_err(TransportError::new)?;
        Self::open_with(&api)
    }

    /// Open the first Mystic Light device using an existing [`HidApi`] context.
    ///
    /// Prefer this when you already hold an [`HidApi`] (creating one scans the
    /// whole bus).
    pub fn open_with(api: &HidApi) -> Result<Self, Error> {
        let device = api
            .open(VENDOR_ID, PRODUCT_ID)
            .map_err(|_| Error::DeviceNotFound {
                vid: VENDOR_ID,
                pid: PRODUCT_ID,
            })?;
        Ok(Self { device })
    }
}

impl Transport for HidTransport {
    fn send_feature_report(&mut self, data: &[u8]) -> Result<(), TransportError> {
        self.device
            .send_feature_report(data)
            .map_err(TransportError::new)
    }
}

/// A connected Mystic Light HID device, as discovered by [`list`].
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// OS HID path (e.g. `/dev/hidraw4` on Linux).
    pub path: String,
    /// Product string reported by the device, if any.
    pub product: Option<String>,
    /// Serial number, if the device exposes one.
    pub serial: Option<String>,
}

/// Enumerate every connected Mystic Light HID interface.
pub fn list() -> Result<Vec<DeviceInfo>, Error> {
    let api = HidApi::new().map_err(TransportError::new)?;
    Ok(list_with(&api))
}

/// Enumerate Mystic Light interfaces using an existing [`HidApi`] context.
pub fn list_with(api: &HidApi) -> Vec<DeviceInfo> {
    api.device_list()
        .filter(|d| d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID)
        .map(|d| DeviceInfo {
            path: d.path().to_string_lossy().into_owned(),
            product: d.product_string().map(str::to_owned),
            serial: d.serial_number().map(str::to_owned),
        })
        .collect()
}
