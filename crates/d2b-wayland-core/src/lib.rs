#![forbid(unsafe_code)]

//! Safe Wayland-facing data types shared by d2b desktop integrations.
//!
//! This crate intentionally models only normal client metadata and presentation
//! colors. It does not parse proxy wire bytes, own file descriptors, or expose
//! ancillary-data transport primitives.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

const MAX_METADATA_LEN: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn css_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn css_hex_with_alpha(self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }
}

impl FromStr for RgbaColor {
    type Err = WaylandCoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix('#')
            .ok_or(WaylandCoreError::InvalidColor)?;
        match hex.len() {
            3 => {
                let r = parse_nibble(hex, 0)?;
                let g = parse_nibble(hex, 1)?;
                let b = parse_nibble(hex, 2)?;
                Ok(Self::rgb(r * 17, g * 17, b * 17))
            }
            6 => Ok(Self::rgb(
                parse_byte(hex, 0)?,
                parse_byte(hex, 2)?,
                parse_byte(hex, 4)?,
            )),
            8 => Ok(Self::rgba(
                parse_byte(hex, 0)?,
                parse_byte(hex, 2)?,
                parse_byte(hex, 4)?,
                parse_byte(hex, 6)?,
            )),
            _ => Err(WaylandCoreError::InvalidColor),
        }
    }
}

fn parse_nibble(hex: &str, index: usize) -> Result<u8, WaylandCoreError> {
    hex.as_bytes()
        .get(index)
        .and_then(|value| char::from(*value).to_digit(16))
        .map(|value| value as u8)
        .ok_or(WaylandCoreError::InvalidColor)
}

fn parse_byte(hex: &str, index: usize) -> Result<u8, WaylandCoreError> {
    u8::from_str_radix(
        hex.get(index..index + 2)
            .ok_or(WaylandCoreError::InvalidColor)?,
        16,
    )
    .map_err(|_| WaylandCoreError::InvalidColor)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPalette {
    pub host: RgbaColor,
    pub environment: RgbaColor,
    pub vm: RgbaColor,
    pub active: RgbaColor,
    pub warning: RgbaColor,
}

impl Default for UiPalette {
    fn default() -> Self {
        Self {
            host: RgbaColor::rgb(0x8a, 0xb4, 0xf8),
            environment: RgbaColor::rgb(0x81, 0xc9, 0x95),
            vm: RgbaColor::rgb(0xfd, 0xd6, 0x63),
            active: RgbaColor::rgb(0xff, 0xff, 0xff),
            warning: RgbaColor::rgb(0xf2, 0x8b, 0x82),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WaylandSurfaceKind {
    Toplevel,
    Popup,
    LayerSurface,
    #[default]
    Unknown,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WaylandMetadataValue(String);

impl WaylandMetadataValue {
    pub fn new(value: impl Into<String>) -> Result<Self, WaylandCoreError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_METADATA_LEN || value.contains('\0') {
            return Err(WaylandCoreError::InvalidMetadata);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for WaylandMetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WaylandMetadataValue")
            .field(&"<metadata>")
            .finish()
    }
}

impl fmt::Display for WaylandMetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for WaylandMetadataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WaylandMetadataValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaylandClientMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<WaylandMetadataValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<WaylandMetadataValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<WaylandMetadataValue>,
}

impl WaylandClientMetadata {
    pub fn metrics_label_value(&self) -> &'static str {
        "wayland-client"
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaylandSurfaceMetadata {
    pub client: WaylandClientMetadata,
    #[serde(default)]
    pub kind: WaylandSurfaceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_name: Option<WaylandMetadataValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WaylandCoreError {
    #[error("invalid color component")]
    InvalidColor,
    #[error("invalid Wayland metadata value")]
    InvalidMetadata,
}

#[cfg(test)]
mod tests {
    use super::{RgbaColor, WaylandClientMetadata, WaylandCoreError, WaylandMetadataValue};
    use std::str::FromStr;

    #[test]
    fn parses_css_hex_colors() {
        assert_eq!(
            RgbaColor::from_str("#abc"),
            Ok(RgbaColor::rgb(0xaa, 0xbb, 0xcc))
        );
        assert_eq!(
            RgbaColor::from_str("#112233"),
            Ok(RgbaColor::rgb(0x11, 0x22, 0x33))
        );
        assert_eq!(
            RgbaColor::from_str("#11223344"),
            Ok(RgbaColor::rgba(0x11, 0x22, 0x33, 0x44))
        );
        assert_eq!(
            RgbaColor::from_str("112233"),
            Err(WaylandCoreError::InvalidColor)
        );
    }

    #[test]
    fn serializes_safe_wayland_client_metadata() {
        let metadata = WaylandClientMetadata {
            app_id: Some(WaylandMetadataValue::new("org.example.App").unwrap()),
            title: Some(WaylandMetadataValue::new("Example").unwrap()),
            display_name: Some(WaylandMetadataValue::new("wayland-1").unwrap()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("org.example.App"));
        assert_eq!(metadata.metrics_label_value(), "wayland-client");
        assert!(!format!("{metadata:?}").contains("org.example.App"));
    }

    #[test]
    fn rejects_nul_and_oversized_metadata_values() {
        assert_eq!(
            WaylandMetadataValue::new("bad\0value").unwrap_err(),
            WaylandCoreError::InvalidMetadata
        );
        assert_eq!(
            WaylandMetadataValue::new("x".repeat(257)).unwrap_err(),
            WaylandCoreError::InvalidMetadata
        );
        assert!(serde_json::from_str::<WaylandMetadataValue>("\"bad\\u0000value\"").is_err());
    }
}
