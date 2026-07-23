#![forbid(unsafe_code)]

use d2b_toolkit_core::ShellName;
use d2b_wayland_core::{UiPalette, WaylandSurfaceMetadata};
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

pub fn shell_module(_shell: &ShellName) -> WaybarModule {
    WaybarModule::new("shell")
        .with_class("d2b-shell")
        .with_tooltip("d2b shell")
}

pub fn palette_summary(palette: &UiPalette) -> WaybarModule {
    WaybarModule::new(palette.vm.css_hex())
        .with_class("d2b-palette")
        .with_tooltip("d2b VM color")
}

pub fn surface_module(surface: &WaylandSurfaceMetadata) -> WaybarModule {
    let mut module =
        WaybarModule::new(surface.client.metrics_label_value()).with_class("d2b-wayland");
    if let Some(scale) = surface.scale {
        module.alt = Some(format!("scale-{scale}"));
    }
    module
}

#[cfg(test)]
mod tests {
    use super::{palette_summary, shell_module, surface_module, WaybarModule};
    use d2b_toolkit_core::ShellName;
    use d2b_wayland_core::{UiPalette, WaylandClientMetadata, WaylandSurfaceMetadata};

    #[test]
    fn shell_name_is_not_rendered_as_label() {
        let module = shell_module(&ShellName::new("private-project-shell").unwrap());
        let encoded = module.to_waybar_json().unwrap();
        assert!(!encoded.contains("private-project-shell"));
        assert!(encoded.contains("d2b-shell"));
    }

    #[test]
    fn serializes_waybar_custom_module_shape() {
        let module = WaybarModule::new("ready")
            .with_class("d2b-ready")
            .with_tooltip("Ready");
        let encoded = module.to_waybar_json().unwrap();
        assert_eq!(
            encoded,
            r#"{"text":"ready","tooltip":"Ready","class":["d2b-ready"]}"#
        );
    }

    #[test]
    fn palette_summary_uses_vm_color() {
        let encoded = palette_summary(&UiPalette::default())
            .to_waybar_json()
            .unwrap();
        assert!(encoded.contains("#fdd663"));
        assert!(encoded.contains("d2b-palette"));
    }

    #[test]
    fn surface_module_uses_bounded_client_label() {
        let surface = WaylandSurfaceMetadata {
            client: WaylandClientMetadata::default(),
            scale: Some(2),
            ..Default::default()
        };
        let encoded = surface_module(&surface).to_waybar_json().unwrap();
        assert!(encoded.contains("wayland-client"));
        assert!(encoded.contains("scale-2"));
    }
}
