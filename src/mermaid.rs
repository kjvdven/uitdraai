use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use directories::ProjectDirs;

/// Blocks past this many in one document stay code blocks; a cheap guard against a hostile file.
pub const MAX_DIAGRAMS: usize = 100;

// Each mmdc run starts a headless Chromium, ~1.5 s on a laptop.
const TIMEOUT: Duration = Duration::from_secs(20);

// `secure` keeps `%%{init}%%` in a diagram from loosening these again.
// Without htmlLabels, labels are SVG text instead of <foreignObject>, which WeasyPrint drops.
const CONFIG: &str = r#"{
  "securityLevel": "strict",
  "htmlLabels": false,
  "flowchart": { "htmlLabels": false },
  "secure": ["secure", "securityLevel", "startOnLoad", "maxTextSize", "maxEdges", "htmlLabels"]
}"#;

// Editing a diagram adds an entry per save; start over rather than grow without bound.
const CACHE_LIMIT: usize = 200;

/// Rendered SVG per diagram source; `None` when mmdc failed or the result looked unsafe.
static CACHE: LazyLock<Mutex<HashMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static AVAILABLE: LazyLock<bool> = LazyLock::new(|| {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join("mmdc").is_file()))
});

/// A Mermaid block as HTML, and whether it still waits for `render_missing`.
pub struct Block {
    pub html: String,
    pub pending: bool,
}

/// The HTML for a ```` ```mermaid ```` block from the cache, or a placeholder until it is rendered.
/// Returns `None` without mmdc; the caller then keeps the plain code block.
pub fn block(source: &str) -> Option<Block> {
    if !*AVAILABLE {
        return None;
    }
    let cache = CACHE.lock().ok()?;
    Some(match cache.get(source) {
        Some(Some(svg)) => Block {
            html: format!("<div class=\"mermaid\">{svg}</div>"),
            pending: false,
        },
        Some(None) => Block {
            html: with_source(
                "mermaid mermaid-failed",
                "<p class=\"mermaid-status\" role=\"alert\">Unable to render this Mermaid diagram</p>",
                source,
            ),
            pending: false,
        },
        None => Block {
            html: "<div class=\"mermaid mermaid-pending\">\
                <p class=\"mermaid-status\" role=\"status\">Rendering diagram…</p></div>"
                .to_owned(),
            pending: true,
        },
    })
}

/// Runs mmdc for each source not in the cache yet. Blocks; call it off the GTK thread.
pub fn render_missing(sources: &[String]) {
    for source in sources {
        let known = CACHE
            .lock()
            .map(|cache| cache.contains_key(source))
            .unwrap_or(true);
        if known {
            continue;
        }
        let svg = match run(source) {
            Ok(svg) if !looks_unsafe(&svg) => Some(svg),
            Ok(_) => {
                eprintln!("Error: mmdc output rejected: it contains scripts, links or HTML");
                None
            }
            Err(err) => {
                eprintln!("Error: {err:#}");
                None
            }
        };
        if let Ok(mut cache) = CACHE.lock() {
            if cache.len() >= CACHE_LIMIT {
                cache.clear();
            }
            cache.insert(source.clone(), svg);
        }
    }
}

/// A failed render keeps the source visible under the status line.
fn with_source(class: &str, status: &str, source: &str) -> String {
    format!(
        "<div class=\"{class}\">{status}<pre><code>{}</code></pre></div>",
        escape(source)
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

static CONFIG_FILE: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    let dir = ProjectDirs::from("io.github", "kjvdven", "uitdraai")?
        .cache_dir()
        .to_owned();
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join("mermaid.json");
    fs::write(&path, CONFIG).ok()?;
    Some(path)
});

fn run(source: &str) -> Result<String> {
    let config = CONFIG_FILE
        .as_ref()
        .context("cannot write the mmdc config to the cache directory")?;
    // mmdc's id also scopes its <style>; without one every diagram is #my-svg and they clash.
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let id = format!("mermaid-{:016x}", hasher.finish());

    let mut child = Command::new("mmdc")
        .args([
            "--quiet",
            "--input",
            "-",
            "--output",
            "-",
            "--outputFormat",
            "svg",
        ])
        .args(["--backgroundColor", "transparent", "--svgId", &id])
        .arg("--configFile")
        .arg(config)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot start mmdc")?;

    let mut stdin = child.stdin.take().context("mmdc has no stdin")?;
    let mut stdout = child.stdout.take().context("mmdc has no stdout")?;
    let input = source.to_owned();
    let writer = thread::spawn(move || stdin.write_all(input.as_bytes()));
    let reader = thread::spawn(move || {
        let mut svg = String::new();
        stdout.read_to_string(&mut svg).map(|_| svg)
    });

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().context("cannot wait for mmdc")? {
            break status;
        }
        if start.elapsed() > TIMEOUT {
            // Not joining the reader: Chromium may still hold the pipe open.
            let _ = child.kill();
            let _ = child.wait();
            bail!("mmdc took longer than {} s", TIMEOUT.as_secs());
        }
        thread::sleep(Duration::from_millis(20));
    };
    if !status.success() {
        bail!("mmdc cannot render this diagram ({status})");
    }
    writer
        .join()
        .map_err(|_| anyhow!("writing to mmdc panicked"))?
        .context("cannot send the diagram to mmdc")?;
    reader
        .join()
        .map_err(|_| anyhow!("reading from mmdc panicked"))?
        .context("cannot read the SVG from mmdc")
}

/// Strict mode should already strip all of this; a check here doesn't depend on mmdc getting it right.
fn looks_unsafe(svg: &str) -> bool {
    let lower = svg.to_ascii_lowercase();
    let markers = [
        "<script",
        "<foreignobject",
        "<image",
        "javascript:",
        "href=",
        "url(http",
        "@import",
    ];
    markers.iter().any(|marker| lower.contains(marker)) || has_event_handler(&lower)
}

/// Finds an attribute like ` onload=`: " on", letters, then "=".
fn has_event_handler(lower: &str) -> bool {
    lower.match_indices(" on").any(|(at, _)| {
        let rest = &lower[at + 3..];
        let name_len = rest.bytes().take_while(u8::is_ascii_alphabetic).count();
        name_len > 0 && rest[name_len..].starts_with('=')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_scripts_links_and_event_handlers() {
        for svg in [
            "<svg><script>alert(1)</script></svg>",
            "<svg><rect onload=\"alert(1)\"/></svg>",
            "<svg><rect ONCLICK=x/></svg>",
            "<svg><a xlink:href=\"https://evil.example\"><text>x</text></a></svg>",
            "<svg><a href=\"javascript:alert(1)\"/></svg>",
            "<svg><foreignObject><div>x</div></foreignObject></svg>",
            "<svg><image href=\"x.png\"/></svg>",
            "<svg><style>@import 'https://evil.example/a.css';</style></svg>",
            "<svg><rect style=\"fill:url(https://evil.example/x)\"/></svg>",
        ] {
            assert!(looks_unsafe(svg), "{svg}");
        }
    }

    #[test]
    fn accepts_plain_drawing_and_text_with_on() {
        let svg = "<svg id=\"m\"><style>#m .node{fill:#fff}</style>\
            <path marker-end=\"url(#m_arrow)\"/><text>turn on the light, one by one</text></svg>";
        assert!(!looks_unsafe(svg));
    }

    #[test]
    fn placeholder_escapes_the_source() {
        let html = with_source("mermaid", "", "A[\"<b>x</b>\"] --> B & C");
        assert!(html.contains("A[&quot;&lt;b&gt;x&lt;/b&gt;&quot;] --&gt; B &amp; C"));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn renders_with_mmdc_when_installed() {
        if !*AVAILABLE {
            eprintln!("skipped: mmdc not installed");
            return;
        }
        let source = "flowchart LR\n  Een --> Twee\n".to_owned();
        assert!(block(&source).is_some_and(|block| block.pending));
        render_missing(std::slice::from_ref(&source));
        let block = block(&source).unwrap();
        assert!(!block.pending);
        assert!(
            block
                .html
                .starts_with("<div class=\"mermaid\"><svg id=\"mermaid-")
        );
        assert!(!block.html.contains("my-svg"));
        assert!(!block.html.contains("<foreignObject"));
    }

    #[test]
    fn hostile_diagram_does_not_render_unsafe_svg() {
        if !*AVAILABLE {
            eprintln!("skipped: mmdc not installed");
            return;
        }
        let source = "flowchart LR\n  A[\"<script>alert(1)</script>\"] --> B\n  \
            click B \"javascript:alert(1)\"\n  linkStyle 0 stroke:red\" onmouseover=\"alert(3)\n"
            .to_owned();
        render_missing(std::slice::from_ref(&source));
        // A failed block shows the escaped source, which still contains the words.
        let html = block(&source).unwrap().html;
        assert!(html.starts_with("<div class=\"mermaid mermaid-failed\">") || !looks_unsafe(&html));
        assert!(!html.contains("<script"));
    }
}
