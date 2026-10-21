use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use anyhow::{Context, Result};
use bezel_mermaid::{Metrics, Options};

/// Blocks past this many in one document stay code blocks; a cheap guard against a hostile file.
pub const MAX_DIAGRAMS: usize = 100;

// Editing a diagram adds an entry per save; start over rather than grow without bound.
const CACHE_LIMIT: usize = 200;

/// Rendered SVG per diagram source; `None` when the render failed or the result looked unsafe.
static CACHE: LazyLock<Mutex<HashMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Pele's defaults: source, output and edge limits included.
static OPTIONS: LazyLock<Options> = LazyLock::new(Options::default);

/// The HTML for a ```` ```mermaid ```` block: the SVG, or the error state with the source.
pub fn block(source: &str) -> String {
    let cached = CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.get(source).cloned());
    let svg = match cached {
        Some(svg) => svg,
        None => {
            let svg = match run(source) {
                Ok(svg) if !looks_unsafe(&svg) => Some(svg),
                Ok(_) => {
                    eprintln!("Error: diagram output rejected: it contains scripts, links or HTML");
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
                cache.insert(source.to_owned(), svg.clone());
            }
            svg
        }
    };
    match svg {
        Some(svg) => format!("<div class=\"mermaid\">{svg}</div>"),
        None => with_source(
            "mermaid mermaid-failed",
            "<p class=\"mermaid-status\" role=\"alert\">Unable to render this Mermaid diagram</p>",
            source,
        ),
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

// `Metrics` measures text with Arial's advances; the page font differs a little, so
// labels can sit slightly loose or tight. Colours and fonts come from the theme's
// `--bezel-*` variables, so the SVG carries no style of its own.
fn run(source: &str) -> Result<String> {
    bezel_mermaid::render(source, &OPTIONS, &Metrics)
        .map(|drawing| drawing.svg)
        .context("cannot render this diagram")
}

/// The renderer escapes labels and drops `javascript:` links; a check here doesn't depend on it.
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
    fn error_state_escapes_the_source() {
        let html = with_source("mermaid", "", "A[\"<b>x</b>\"] --> B & C");
        assert!(html.contains("A[&quot;&lt;b&gt;x&lt;/b&gt;&quot;] --&gt; B &amp; C"));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn renders_a_flowchart_without_its_own_style() {
        let html = block("flowchart LR\n  Een --> Twee\n");
        assert!(html.starts_with("<div class=\"mermaid\"><svg "), "{html}");
        assert!(html.contains("class=\"bezel bezel-flowchart\""));
        assert!(html.contains("var(--bezel-fg"));
        assert!(!html.contains("<style"));
        assert!(!html.contains("<foreignObject"));
        assert!(html.contains("Twee"));
    }

    #[test]
    fn broken_diagram_shows_the_error_state() {
        let html = block("flowchart LR\n  A -->\n");
        assert!(
            html.starts_with("<div class=\"mermaid mermaid-failed\">"),
            "{html}"
        );
        assert!(html.contains("A --&gt;"));
    }

    #[test]
    fn hostile_diagram_does_not_render_unsafe_svg() {
        let html = block(
            "%%{init: {\"securityLevel\": \"loose\", \"flowchart\": {\"htmlLabels\": true}}}%%\n\
            flowchart LR\n  A[\"<script>alert(1)</script>\"] --> B[\"<img src='https://evil.example/x.png'>\"]\n  \
            click B \"javascript:alert(1)\"\n  linkStyle 0 stroke:red\" onmouseover=\"alert(3)\n",
        );
        // A failed block shows the escaped source, which still contains the words.
        assert!(html.starts_with("<div class=\"mermaid mermaid-failed\">") || !looks_unsafe(&html));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<img"));
    }
}
