#![forbid(unsafe_code)]

//! Presentation-only color parsing for d2b desktop clients.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    type Err = ColorArtifactError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix('#')
            .ok_or(ColorArtifactError::InvalidColor)?;
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
            _ => Err(ColorArtifactError::InvalidColor),
        }
    }
}

fn parse_nibble(hex: &str, index: usize) -> Result<u8, ColorArtifactError> {
    hex.as_bytes()
        .get(index)
        .and_then(|value| char::from(*value).to_digit(16))
        .map(|value| value as u8)
        .ok_or(ColorArtifactError::InvalidColor)
}

fn parse_byte(hex: &str, index: usize) -> Result<u8, ColorArtifactError> {
    u8::from_str_radix(
        hex.get(index..index + 2)
            .ok_or(ColorArtifactError::InvalidColor)?,
        16,
    )
    .map_err(|_| ColorArtifactError::InvalidColor)
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
pub struct UiColorsArtifact {
    pub palette: UiPalette,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUiColorsArtifact {
    pub artifact: UiColorsArtifact,
    pub fallbacks: Vec<ColorFallback>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorFallback {
    pub role: ColorRole,
    pub reason: FallbackReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    Host,
    Environment,
    Vm,
    Active,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    Missing,
    Invalid,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorArtifactError {
    #[error("failed to decode ui colors artifact")]
    Decode(#[from] serde_json::Error),
    #[error("invalid color component")]
    InvalidColor,
}

pub fn parse_ui_colors_json(input: &str) -> Result<UiColorsArtifact, ColorArtifactError> {
    parse_ui_colors_json_with_fallbacks(input).map(|parsed| parsed.artifact)
}

pub fn parse_ui_colors_json_with_fallbacks(
    input: &str,
) -> Result<ParsedUiColorsArtifact, ColorArtifactError> {
    let raw: RawUiColorsArtifact = serde_json::from_str(input)?;
    Ok(raw.into_parsed())
}

pub fn css_variables(palette: &UiPalette) -> String {
    [
        ("d2b-host", palette.host),
        ("d2b-env", palette.environment),
        ("d2b-vm", palette.vm),
        ("d2b-active", palette.active),
        ("d2b-warning", palette.warning),
    ]
    .into_iter()
    .map(|(name, color)| format!("@define-color {name} {};", color.css_hex()))
    .collect::<Vec<_>>()
    .join("\n")
}

#[derive(Debug, Default, Deserialize)]
struct RawUiColorsArtifact {
    #[serde(default)]
    palette: Option<RawPalette>,
    #[serde(default)]
    host: Option<RawColor>,
    #[serde(default, alias = "env")]
    environment: Option<RawColor>,
    #[serde(default)]
    vm: Option<RawColor>,
    #[serde(default)]
    active: Option<RawColor>,
    #[serde(default)]
    warning: Option<RawColor>,
}

impl RawUiColorsArtifact {
    fn into_parsed(self) -> ParsedUiColorsArtifact {
        let defaults = UiPalette::default();
        let palette = self.palette.unwrap_or_default();
        let mut fallbacks = Vec::new();
        let host = color_or_default(
            ColorRole::Host,
            self.host.or(palette.host),
            defaults.host,
            &mut fallbacks,
        );
        let environment = color_or_default(
            ColorRole::Environment,
            self.environment.or(palette.environment),
            defaults.environment,
            &mut fallbacks,
        );
        let vm = color_or_default(
            ColorRole::Vm,
            self.vm.or(palette.vm),
            defaults.vm,
            &mut fallbacks,
        );
        let active = color_or_default(
            ColorRole::Active,
            self.active.or(palette.active),
            defaults.active,
            &mut fallbacks,
        );
        let warning = color_or_default(
            ColorRole::Warning,
            self.warning.or(palette.warning),
            defaults.warning,
            &mut fallbacks,
        );
        ParsedUiColorsArtifact {
            artifact: UiColorsArtifact {
                palette: UiPalette {
                    host,
                    environment,
                    vm,
                    active,
                    warning,
                },
            },
            fallbacks,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawPalette {
    #[serde(default)]
    host: Option<RawColor>,
    #[serde(default, alias = "env")]
    environment: Option<RawColor>,
    #[serde(default)]
    vm: Option<RawColor>,
    #[serde(default)]
    active: Option<RawColor>,
    #[serde(default)]
    warning: Option<RawColor>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawColor {
    Hex(String),
    Components {
        r: u8,
        g: u8,
        b: u8,
        #[serde(default = "opaque_alpha")]
        a: u8,
    },
}

impl RawColor {
    fn into_color(self) -> Result<RgbaColor, ColorArtifactError> {
        match self {
            Self::Hex(value) => RgbaColor::from_str(&value),
            Self::Components { r, g, b, a } => Ok(RgbaColor::rgba(r, g, b, a)),
        }
    }
}

const fn opaque_alpha() -> u8 {
    255
}

fn color_or_default(
    role: ColorRole,
    color: Option<RawColor>,
    default: RgbaColor,
    fallbacks: &mut Vec<ColorFallback>,
) -> RgbaColor {
    match color {
        Some(color) => match color.into_color() {
            Ok(color) => color,
            Err(_) => {
                fallbacks.push(ColorFallback {
                    role,
                    reason: FallbackReason::Invalid,
                });
                default
            }
        },
        None => {
            fallbacks.push(ColorFallback {
                role,
                reason: FallbackReason::Missing,
            });
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colors_and_reports_fallbacks() {
        let parsed = parse_ui_colors_json_with_fallbacks(
            r##"{
              "palette": {
                "host": "#112233",
                "environment": "#abc",
                "vm": { "r": 1, "g": 2, "b": 3 },
                "active": "#01020304",
                "warning": "not-a-color"
              }
            }"##,
        )
        .unwrap();

        assert_eq!(
            parsed.artifact.palette.host,
            RgbaColor::rgb(0x11, 0x22, 0x33)
        );
        assert_eq!(
            parsed.artifact.palette.environment,
            RgbaColor::rgb(0xaa, 0xbb, 0xcc)
        );
        assert_eq!(parsed.artifact.palette.vm, RgbaColor::rgb(1, 2, 3));
        assert_eq!(parsed.artifact.palette.active, RgbaColor::rgba(1, 2, 3, 4));
        assert_eq!(
            parsed.fallbacks,
            vec![ColorFallback {
                role: ColorRole::Warning,
                reason: FallbackReason::Invalid,
            }]
        );
    }

    #[test]
    fn emits_stable_css_names() {
        let css = css_variables(&UiPalette::default());
        assert!(css.contains("@define-color d2b-host"));
        assert!(css.contains("@define-color d2b-vm"));
    }
}
