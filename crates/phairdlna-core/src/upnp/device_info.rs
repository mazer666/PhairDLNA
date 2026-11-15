//! The identity strings of the device description.
//!
//! What: the names and numbers the device description carries — the friendly
//!       name the user chose and the product strings of the build.
//! Why: the friendly name is the only value the user controls, and it reaches
//!       the XML, so it has to be escaped there (`INT-027`). The product
//!       strings are not fixed yet; keeping them in one struct makes them
//!       configurable and keeps the description and the discovery answer from
//!       disagreeing about them (`INT-012`).
//! Example: `DeviceInfo::new("Living room")` builds the identity with the
//!       project defaults for every product string.

use crate::{PROJECT_NAME, PROJECT_VERSION};

/// The URL of the project, used as the manufacturer and model address.
pub const PROJECT_URL: &str = "https://github.com/mazer666/PhairDLNA";

/// The default model name of the server.
///
/// The product strings are not fixed yet; they are configurable, and this is
/// the value the description falls back to.
pub const DEFAULT_MODEL_NAME: &str = "PhairDLNA Media Server";

/// The default model description of the server.
pub const DEFAULT_MODEL_DESCRIPTION: &str = "A media server that shares the folders you choose.";

/// The identity strings the device description carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// The name the user chose and the client shows.
    pub friendly_name: String,
    /// The name of the manufacturer.
    pub manufacturer: String,
    /// The address of the manufacturer.
    pub manufacturer_url: String,
    /// A sentence about the model.
    pub model_description: String,
    /// The name of the model.
    pub model_name: String,
    /// The number of the model, which identifies the build.
    pub model_number: String,
    /// The address of the model.
    pub model_url: String,
    /// The number of the device, which identifies the installation.
    pub serial_number: String,
}

impl DeviceInfo {
    /// Builds the identity with the project defaults for every product string.
    ///
    /// Only the friendly name is the user's; the product strings are the
    /// project's, and the caller may change them because they are not fixed
    /// yet.
    pub fn new(friendly_name: impl Into<String>) -> Self {
        Self {
            friendly_name: friendly_name.into(),
            manufacturer: PROJECT_NAME.to_owned(),
            manufacturer_url: PROJECT_URL.to_owned(),
            model_description: DEFAULT_MODEL_DESCRIPTION.to_owned(),
            model_name: DEFAULT_MODEL_NAME.to_owned(),
            model_number: PROJECT_VERSION.to_owned(),
            model_url: PROJECT_URL.to_owned(),
            serial_number: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceInfo;

    /// The defaults carry the project name and the build number.
    #[test]
    fn the_defaults_carry_the_project_strings() {
        let info = DeviceInfo::new("Living room");
        assert_eq!(info.friendly_name, "Living room");
        assert_eq!(info.manufacturer, crate::PROJECT_NAME);
        assert_eq!(info.model_number, crate::PROJECT_VERSION);
        assert!(!info.model_description.is_empty());
    }

    /// A field can be changed, because the product strings are configurable.
    #[test]
    fn a_product_string_can_be_changed() {
        let mut info = DeviceInfo::new("Living room");
        info.model_name = "Living room server".to_owned();
        assert_eq!(info.model_name, "Living room server");
    }
}
