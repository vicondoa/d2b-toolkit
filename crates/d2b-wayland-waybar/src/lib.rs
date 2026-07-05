#![forbid(unsafe_code)]

use d2b_toolkit_core::ShellName;
use d2b_wayland_core::UiPalette;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaybarModule {
    pub name: String,
    pub text: String,
    pub class: String,
}

pub fn shell_module(_shell: &ShellName) -> WaybarModule {
    WaybarModule {
        name: "d2b-shell".into(),
        text: "shell".into(),
        class: "d2b-shell".into(),
    }
}

pub fn palette_summary(palette: &UiPalette) -> WaybarModule {
    WaybarModule {
        name: "d2b-palette".into(),
        text: palette.vm.css_hex(),
        class: "d2b-palette".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::shell_module;
    use d2b_toolkit_core::ShellName;

    #[test]
    fn shell_name_is_not_rendered_as_label() {
        let module = shell_module(&ShellName::new("private-project-shell"));
        let encoded = serde_json::to_string(&module).unwrap();
        assert!(!encoded.contains("private-project-shell"));
        assert!(encoded.contains("d2b-shell"));
    }
}
