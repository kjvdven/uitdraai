use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;

/// Written when the user opens a config file that does not exist yet.
#[cfg_attr(
    not(feature = "gui"),
    allow(dead_code, reason = "only the preview window uses it")
)]
const TEMPLATE: &str = r#"# uitdraai configuration. Every key is optional; command-line flags win.

# Theme used when --theme and --css are not given.
# theme = "default"

# Where PDFs go by default, instead of next to the Markdown file.
# export_dir = "/home/you/Documents"

# Command for "Open in editor"; the file is appended. Default: the desktop's app for Markdown.
# editor = ["ghostty", "-e", "hx"]

# Set to true if the preview window stays empty (common on NVIDIA).
# disable_dmabuf = false
"#;

/// Settings from `~/.config/uitdraai/config.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub theme: Option<String>,
    pub export_dir: Option<PathBuf>,
    #[cfg_attr(
        not(feature = "gui"),
        allow(dead_code, reason = "only the preview window uses it")
    )]
    pub editor: Option<Vec<String>>,
    #[serde(default)]
    #[cfg_attr(
        not(feature = "gui"),
        allow(dead_code, reason = "only the preview window uses it")
    )]
    pub disable_dmabuf: bool,
}

/// Location of the config file, `~/.config/uitdraai/config.toml` on Linux.
pub fn path() -> Option<PathBuf> {
    ProjectDirs::from("io.github", "kjvdven", "uitdraai")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

/// Reads the config at `path`; a missing file gives the defaults.
pub fn load(path: Option<&Path>) -> Result<Config> {
    let Some(path) = path else {
        return Ok(Config::default());
    };
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Config::default()),
        Err(err) => return Err(err).with_context(|| format!("cannot read {}", path.display())),
    };
    toml::from_str(&text).with_context(|| format!("invalid config in {}", path.display()))
}

/// Creates the config file from the commented template unless it already exists.
#[cfg_attr(
    not(feature = "gui"),
    allow(dead_code, reason = "only the preview window uses it")
)]
pub fn ensure_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }
    fs::write(path, TEMPLATE).with_context(|| format!("cannot write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uitdraai-config-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_gives_defaults() {
        let config = load(Some(&temp_dir("missing").join("config.toml"))).unwrap();
        assert!(config.theme.is_none());
        assert!(!config.disable_dmabuf);
    }

    #[test]
    fn reads_all_keys() {
        let path = temp_dir("all").join("config.toml");
        fs::write(
            &path,
            "theme = \"sepia\"\nexport_dir = \"/tmp/pdf\"\neditor = [\"hx\"]\ndisable_dmabuf = true\n",
        )
        .unwrap();
        let config = load(Some(&path)).unwrap();
        assert_eq!(config.theme.as_deref(), Some("sepia"));
        assert_eq!(config.export_dir, Some(PathBuf::from("/tmp/pdf")));
        assert_eq!(config.editor, Some(vec!["hx".to_owned()]));
        assert!(config.disable_dmabuf);
    }

    #[test]
    fn typo_in_a_key_is_an_error() {
        let path = temp_dir("typo").join("config.toml");
        fs::write(&path, "thme = \"sepia\"\n").unwrap();
        let err = format!("{:#}", load(Some(&path)).unwrap_err());
        assert!(err.contains("thme"), "{err}");
    }

    #[test]
    fn template_is_valid_and_not_overwritten() {
        let path = temp_dir("template").join("sub").join("config.toml");
        ensure_exists(&path).unwrap();
        assert!(load(Some(&path)).is_ok());

        fs::write(&path, "theme = \"mine\"\n").unwrap();
        ensure_exists(&path).unwrap();
        assert_eq!(load(Some(&path)).unwrap().theme.as_deref(), Some("mine"));
    }
}
