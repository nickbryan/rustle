use serde::Deserialize;

use crate::input::ModeKeyBindingMap;

/// The `Config` struct represents the top-level configuration structure.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub editor: EditorOptions,
    pub bindings: ModeKeyBindingMap,
}

// This macro reads the contents of the file and places it into the
// compiled binary as a static string slice (&'static str).
const DEFAULT_CONFIG_TOML: &str = include_str!("defaults/config.toml");

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG_TOML).expect("Failed to load default config.")
    }
}

/// The `EditorOptions` struct represents the configuration options for the editor.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EditorOptions {
    pub idle_timeout: u64,
}
