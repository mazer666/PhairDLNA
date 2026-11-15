//! The device description of the server.
//!
//! What: the XML document at `/description.xml` that tells a control point
//!       what this server is and what it can do (`INT-020`, `INT-021`).
//! Why: the control point fetches this document from the `LOCATION` header of
//!       the discovery answer and reads the device type, the identity and the
//!       service list out of it. A document that names a service the server
//!       does not have produces a failure that looks like a client bug, so
//!       the document is generated from the same service table the server
//!       answers from.
//! Example: `DeviceDescription::new` builds the document, and `to_xml`
//!       renders it.

use crate::ssdp::DEVICE_TYPE_MEDIA_SERVER;

use super::device_model::BaseUrl;
use super::device_model::DeviceInfo;
use super::error::UpnpError;
use super::services::{advertised_services, ServiceDefinition};
use super::xml_writer::XmlWriter;

/// The namespace of a device description document.
pub const DEVICE_NAMESPACE: &str = "urn:schemas-upnp-org:device-1-0";

/// The device description of the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescription {
    base_url: BaseUrl,
    udn: String,
    info: DeviceInfo,
}

impl DeviceDescription {
    /// Builds the description of a device.
    ///
    /// The unique device name must be a UUID, because the discovery layer
    /// derives the `USN` from it and the two must match (`INT-044`). The
    /// fields that the specification requires must not be empty: a description
    /// with an empty model name is refused here rather than sent to a client
    /// that would reject it (`CFG-002`).
    pub fn new(base_url: &BaseUrl, udn: &str, info: DeviceInfo) -> Result<Self, UpnpError> {
        let udn = canonical_udn(udn)?;
        let mut info = checked_info(info)?;
        if info.serial_number.trim().is_empty() {
            // The unique device name identifies the installation, so it is the
            // serial number of the device unless the configuration names one.
            info.serial_number = udn.trim_start_matches("uuid:").to_owned();
        }
        Ok(Self {
            base_url: base_url.clone(),
            udn,
            info,
        })
    }

    /// The origin the description is served from.
    pub fn base_url(&self) -> &BaseUrl {
        &self.base_url
    }

    /// The unique device name, with the `uuid:` prefix.
    pub fn udn(&self) -> &str {
        &self.udn
    }

    /// The identity strings of the description.
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// The services the description advertises.
    pub fn services(&self) -> [ServiceDefinition; 2] {
        advertised_services()
    }

    /// The description as an XML document.
    pub fn to_xml(&self) -> String {
        let mut writer = XmlWriter::new();
        writer.declaration();
        writer.open("root", &[("xmlns", DEVICE_NAMESPACE)]);
        writer.open("specVersion", &[]);
        writer.text("major", "1");
        writer.text("minor", "0");
        writer.close("specVersion");
        writer.open("device", &[]);
        self.write_identity(&mut writer);
        self.write_services(&mut writer);
        writer.close("device");
        writer.close("root");
        writer.finish()
    }

    /// Writes the identity elements of the device.
    fn write_identity(&self, writer: &mut XmlWriter) {
        writer.text("deviceType", DEVICE_TYPE_MEDIA_SERVER);
        writer.text("friendlyName", &self.info.friendly_name);
        writer.text("manufacturer", &self.info.manufacturer);
        writer.text("manufacturerURL", &self.info.manufacturer_url);
        if !self.info.model_description.is_empty() {
            writer.text("modelDescription", &self.info.model_description);
        }
        writer.text("modelName", &self.info.model_name);
        writer.text("modelNumber", &self.info.model_number);
        writer.text("modelURL", &self.info.model_url);
        writer.text("serialNumber", &self.info.serial_number);
        writer.text("UDN", &self.udn);
    }

    /// Writes the service list of the device.
    fn write_services(&self, writer: &mut XmlWriter) {
        writer.open("serviceList", &[]);
        for service in self.services() {
            writer.open("service", &[]);
            writer.text("serviceType", service.service_type);
            writer.text("serviceId", service.service_id);
            writer.text("SCPDURL", &self.base_url.url(service.scpd_path));
            writer.text("controlURL", &self.base_url.url(service.control_path));
            writer.text("eventSubURL", &self.base_url.url(service.event_path));
            writer.close("service");
        }
        writer.close("serviceList");
    }
}

/// Normalises the unique device name into the `uuid:<UUID>` form.
fn canonical_udn(udn: &str) -> Result<String, UpnpError> {
    let bare = udn.trim().strip_prefix("uuid:").unwrap_or(udn.trim());
    match uuid::Uuid::parse_str(bare) {
        Ok(parsed) => Ok(format!("uuid:{parsed}")),
        Err(_) => Err(UpnpError::InvalidUdn {
            value: udn.to_owned(),
        }),
    }
}

/// Checks that every required identity field carries a value.
fn checked_info(info: DeviceInfo) -> Result<DeviceInfo, UpnpError> {
    let required: [(&'static str, &str); 6] = [
        ("friendly_name", &info.friendly_name),
        ("manufacturer", &info.manufacturer),
        ("manufacturer_url", &info.manufacturer_url),
        ("model_name", &info.model_name),
        ("model_number", &info.model_number),
        ("model_url", &info.model_url),
    ];
    for (field, value) in required {
        if value.trim().is_empty() {
            return Err(UpnpError::MissingField { field });
        }
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::{BaseUrl, DeviceDescription, DeviceInfo};
    use crate::upnp::UpnpError;

    /// The base URL used by the tests.
    fn base() -> BaseUrl {
        BaseUrl::new("http://192.168.1.20:8200").expect("a valid base URL")
    }

    /// The description used by the tests.
    fn description() -> DeviceDescription {
        DeviceDescription::new(
            &base(),
            "11111111-2222-3333-4444-555555555555",
            DeviceInfo::new("Living room"),
        )
        .expect("a complete description")
    }

    /// The description carries the identity elements the specification names.
    #[test]
    fn the_description_carries_the_identity_elements() {
        let xml = description().to_xml();
        for element in [
            "deviceType",
            "friendlyName",
            "manufacturer",
            "modelName",
            "modelNumber",
            "modelURL",
            "serialNumber",
            "UDN",
        ] {
            assert!(
                xml.contains(&format!("<{element}>")),
                "{element} is missing"
            );
        }
        assert!(xml.contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"));
        assert!(xml.contains("<UDN>uuid:11111111-2222-3333-4444-555555555555</UDN>"));
    }

    /// The serial number identifies the installation when the configuration
    /// does not name one.
    #[test]
    fn the_serial_number_falls_back_to_the_unique_device_name() {
        let xml = description().to_xml();
        assert!(xml.contains("<serialNumber>11111111-2222-3333-4444-555555555555</serialNumber>"));
    }

    /// A serial number from the configuration is used as it is.
    #[test]
    fn a_configured_serial_number_is_used() {
        let mut info = DeviceInfo::new("Living room");
        info.serial_number = "SN-4711".to_owned();
        let description =
            DeviceDescription::new(&base(), "11111111-2222-3333-4444-555555555555", info)
                .expect("a complete description");
        assert!(description
            .to_xml()
            .contains("<serialNumber>SN-4711</serialNumber>"));
    }

    /// The description holds exactly one device element and no base URL
    /// element (`INT-045`).
    #[test]
    fn the_description_holds_one_device_and_no_url_base() {
        let xml = description().to_xml();
        assert_eq!(xml.matches("<device>").count(), 1);
        assert!(!xml.contains("<URLBase"));
    }

    /// Both mandatory services are listed, each with a complete set of URLs.
    #[test]
    fn both_services_are_listed() {
        let xml = description().to_xml();
        assert!(xml.contains("urn:schemas-upnp-org:service:ContentDirectory:1"));
        assert!(xml.contains("urn:schemas-upnp-org:service:ConnectionManager:1"));
        assert!(xml.contains("urn:upnp-org:serviceId:ContentDirectory"));
        assert!(xml.contains("urn:upnp-org:serviceId:ConnectionManager"));
        assert!(xml.contains("<SCPDURL>http://192.168.1.20:8200/ContentDirectory.xml</SCPDURL>"));
        assert!(xml.contains("<controlURL>http://192.168.1.20:8200/ctl/CD</controlURL>"));
        assert!(xml.contains("<eventSubURL>http://192.168.1.20:8200/evt/CM</eventSubURL>"));
    }

    /// The description carries neither a DOCTYPE nor a renderer-side service.
    #[test]
    fn the_description_carries_no_doctype_and_no_renderer_service() {
        let xml = description().to_xml();
        assert!(!xml.contains("<!DOCTYPE"));
        assert!(!xml.contains("RenderingControl"));
        assert!(!xml.contains("AVTransport"));
    }

    /// A name with the reserved characters is escaped, and no element is
    /// created from it (`INT-027`).
    #[test]
    fn a_reserved_character_in_the_name_is_escaped() {
        let description = DeviceDescription::new(
            &base(),
            "11111111-2222-3333-4444-555555555555",
            DeviceInfo::new("Living & <room>"),
        )
        .expect("a complete description");
        let xml = description.to_xml();
        assert!(xml.contains("<friendlyName>Living &amp; &lt;room&gt;</friendlyName>"));
        assert!(!xml.contains("<room>"));
    }

    /// A unique device name that is not a UUID is refused.
    #[test]
    fn a_unique_device_name_that_is_not_a_uuid_is_refused() {
        let result = DeviceDescription::new(&base(), "not-a-uuid", DeviceInfo::new("Room"));
        assert_eq!(
            result,
            Err(UpnpError::InvalidUdn {
                value: "not-a-uuid".to_owned()
            })
        );
    }

    /// A required field that is empty is refused, and the error names it.
    #[test]
    fn an_empty_required_field_is_refused() {
        let mut info = DeviceInfo::new("Room");
        info.model_name = String::new();
        let result = DeviceDescription::new(&base(), "11111111-2222-3333-4444-555555555555", info);
        assert_eq!(
            result,
            Err(UpnpError::MissingField {
                field: "model_name"
            })
        );
    }
}
