#![forbid(unsafe_code)]

use d2b_wayland_core::{RgbaColor, UiPalette};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiColorsArtifact {
    pub palette: UiPalette,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorArtifactError {
    #[error("failed to decode ui colors artifact")]
    Decode(#[from] serde_json::Error),
}

pub fn parse_ui_colors_json(input: &str) -> Result<UiColorsArtifact, ColorArtifactError> {
    serde_json::from_str(input).map_err(ColorArtifactError::Decode)
}

pub fn css_variables(palette: &UiPalette) -> String {
    let entries = [
        ("d2b-host", palette.host),
        ("d2b-env", palette.environment),
        ("d2b-vm", palette.vm),
        ("d2b-active", palette.active),
        ("d2b-warning", palette.warning),
    ];
    entries
        .into_iter()
        .map(|(name, color): (&str, RgbaColor)| {
            format!("@define-color {name} {};", color.css_hex())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{css_variables, UiColorsArtifact};

    #[test]
    fn emits_known_css_names() {
        let css = css_variables(&UiColorsArtifact::default().palette);
        assert!(css.contains("@define-color d2b-host"));
        assert!(css.contains("@define-color d2b-vm"));
    }
}
