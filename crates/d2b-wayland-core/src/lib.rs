#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

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

    pub fn css_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
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

#[derive(Debug, thiserror::Error)]
pub enum WaylandCoreError {
    #[error("invalid color component")]
    InvalidColor,
}
