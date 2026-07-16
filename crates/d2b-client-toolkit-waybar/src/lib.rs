#![forbid(unsafe_code)]

//! Generic Waybar presentation helpers with no daemon wire types.

use d2b_client_toolkit_colors::UiPalette;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaybarModule {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub class: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<u8>,
}

impl WaybarModule {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alt: None,
            tooltip: None,
            class: Vec::new(),
            percentage: None,
        }
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class.push(class.into());
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn to_waybar_json(&self) -> Result<String, WaybarError> {
        serde_json::to_string(self).map_err(WaybarError::Encode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WaybarError {
    #[error("failed to encode Waybar module")]
    Encode(serde_json::Error),
}

pub fn palette_summary(palette: &UiPalette) -> WaybarModule {
    WaybarModule::new(palette.vm.css_hex())
        .with_class("d2b-palette")
        .with_tooltip("d2b workload color")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_waybar_custom_module_shape() {
        let encoded = WaybarModule::new("ready")
            .with_class("d2b-ready")
            .with_tooltip("Ready")
            .to_waybar_json()
            .unwrap();
        assert_eq!(
            encoded,
            r#"{"text":"ready","tooltip":"Ready","class":["d2b-ready"]}"#
        );
    }

    #[test]
    fn palette_summary_uses_workload_color() {
        let encoded = palette_summary(&UiPalette::default())
            .to_waybar_json()
            .unwrap();
        assert!(encoded.contains("#fdd663"));
        assert!(encoded.contains("d2b-palette"));
    }
}
