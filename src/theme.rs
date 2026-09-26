use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

const PRINT_CSS: &str = include_str!("../themes/print.css");
const BUILTIN: &[(&str, &str)] = &[("default", include_str!("../themes/default.css"))];
const DEFAULT_THEME: &str = "default";

/// Directory with user themes, `~/.config/uitdraai/themes` on Linux.
pub fn user_theme_dir() -> Option<PathBuf> {
    ProjectDirs::from("io.github", "kjvdven", "uitdraai")
        .map(|dirs| dirs.config_dir().join("themes"))
}

/// Returns the stylesheet for a page: `print.css` followed by the chosen theme.
///
/// `css` (from `--css`) wins over `theme`; without either the default theme is used.
pub fn resolve(
    css: Option<&Path>,
    theme: Option<&str>,
    theme_dir: Option<&Path>,
) -> Result<String> {
    let theme_css = match css {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("cannot read stylesheet {}", path.display()))?,
        None => load(theme.unwrap_or(DEFAULT_THEME), theme_dir)?,
    };
    Ok(format!("{PRINT_CSS}\n{theme_css}"))
}

/// Returns one theme's CSS without `print.css`; user themes shadow built-in ones.
pub fn load(name: &str, theme_dir: Option<&Path>) -> Result<String> {
    // Keeps `--theme ../../x` from reading files outside the theme directory.
    if name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
        bail!("invalid theme name '{name}'");
    }
    if let Some(dir) = theme_dir {
        let path = dir.join(format!("{name}.css"));
        match fs::read_to_string(&path) {
            Ok(css) => return Ok(css),
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("cannot read theme {}", path.display()));
            }
        }
    }
    BUILTIN
        .iter()
        .find(|(builtin, _)| *builtin == name)
        .map(|(_, css)| (*css).to_owned())
        .with_context(|| format!("unknown theme '{name}'"))
}

/// Lists built-in and user theme names, sorted and without duplicates.
pub fn list(theme_dir: Option<&Path>) -> Result<Vec<String>> {
    let mut names: Vec<String> = BUILTIN.iter().map(|(name, _)| (*name).to_owned()).collect();
    if let Some(dir) = theme_dir {
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    let path = entry
                        .with_context(|| format!("cannot read {}", dir.display()))?
                        .path();
                    if path.extension().is_some_and(|ext| ext == "css")
                        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    {
                        names.push(stem.to_owned());
                    }
                }
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("cannot read {}", dir.display()));
            }
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("uitdraai-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn defaults_to_builtin_theme_after_print_css() {
        let css = resolve(None, None, None).unwrap();
        assert!(css.starts_with(PRINT_CSS));
        assert!(css.ends_with(BUILTIN[0].1));
    }

    #[test]
    fn user_theme_shadows_builtin() {
        let dir = temp_dir("shadow");
        fs::write(dir.join("default.css"), "/* mine */").unwrap();
        assert_eq!(load("default", Some(&dir)).unwrap(), "/* mine */");
    }

    #[test]
    fn css_flag_wins_over_theme() {
        let dir = temp_dir("cssflag");
        let path = dir.join("own.css");
        fs::write(&path, "/* own */").unwrap();
        let css = resolve(Some(&path), Some("default"), None).unwrap();
        assert!(css.ends_with("/* own */"));
    }

    #[test]
    fn rejects_unknown_and_unsafe_names() {
        assert!(load("nope", None).is_err());
        assert!(load("../etc/passwd", None).is_err());
        assert!(load("..", None).is_err());
        assert!(load("", None).is_err());
    }

    #[test]
    fn lists_builtin_and_user_themes() {
        let dir = temp_dir("list");
        fs::write(dir.join("zen.css"), "").unwrap();
        fs::write(dir.join("default.css"), "").unwrap();
        fs::write(dir.join("notes.txt"), "").unwrap();
        assert_eq!(list(Some(&dir)).unwrap(), ["default", "zen"]);
    }

    #[test]
    fn user_theme_dir_is_under_config() {
        let dir = user_theme_dir().unwrap();
        assert!(dir.ends_with("uitdraai/themes"));
    }

    #[test]
    fn missing_theme_dir_is_fine() {
        let dir = temp_dir("missing").join("nope");
        assert_eq!(list(Some(&dir)).unwrap(), ["default"]);
        assert!(load("default", Some(&dir)).is_ok());
    }
}
