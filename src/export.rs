use std::ffi::OsString;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};

const WEASYPRINT_MISSING: &str = "weasyprint not found, needed for PDF export.
Install it with your package manager:
  Arch:          sudo pacman -S python-weasyprint
  Fedora:        sudo dnf install weasyprint
  Debian/Ubuntu: sudo apt install weasyprint
  NixOS:         add `weasyprint` to your packages";

/// Writes `html` to `output` as PDF with WeasyPrint.
///
/// Relative links and images resolve against `base_dir`, the Markdown file's directory.
/// Remote URLs are only fetched with `allow_remote`, matching the page's CSP.
pub fn export_pdf(html: &str, base_dir: &Path, output: &Path, allow_remote: bool) -> Result<()> {
    // Trailing slash, or the last path segment is treated as a file and dropped.
    let mut base_url = OsString::from(base_dir);
    base_url.push("/");

    let protocols = if allow_remote {
        "file,data,https"
    } else {
        "file,data"
    };

    let mut child = Command::new("weasyprint")
        .arg("--base-url")
        .arg(&base_url)
        .arg("--allowed-protocols")
        .arg(protocols)
        .arg("--")
        .arg("-")
        .arg(output)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| match err.kind() {
            ErrorKind::NotFound => anyhow!(WEASYPRINT_MISSING),
            _ => anyhow!(err).context("cannot start weasyprint"),
        })?;

    let mut stdin = child.stdin.take().context("weasyprint has no stdin")?;
    // Write from a thread so a chatty stderr can't deadlock against a full stdin pipe.
    let (written, finished) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(html.as_bytes()));
        let finished = child.wait_with_output();
        (writer.join(), finished)
    });
    let finished = finished.context("weasyprint did not finish")?;
    if !finished.status.success() {
        bail!(
            "weasyprint failed ({}):\n{}",
            finished.status,
            String::from_utf8_lossy(&finished.stderr).trim_end()
        );
    }
    written
        .map_err(|_| anyhow!("writing HTML to weasyprint panicked"))?
        .context("cannot send HTML to weasyprint")
}
