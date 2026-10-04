use std::sync::LazyLock;

use comrak::nodes::{NodeMath, NodeValue};
use comrak::options::{Plugins, RenderPlugins};
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Anchorizer, Arena, Options, format_html_with_plugins, parse_document};

/// Switches that loosen the default security settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOptions {
    /// Pass raw HTML in the Markdown through (`--allow-html`).
    pub allow_html: bool,
    /// Allow remote images in the page (`--allow-remote`).
    pub allow_remote: bool,
}

/// A heading in the document, with the `id` its HTML element carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub id: String,
}

/// Rendered HTML plus the headings in it, for the outline.
#[derive(Debug, Clone, Default)]
pub struct Rendered {
    pub html: String,
    pub headings: Vec<Heading>,
    /// Mermaid sources shown as a placeholder until `mermaid::render_missing` has run.
    pub diagrams: Vec<String>,
}

static ADAPTER: LazyLock<SyntectAdapter> =
    LazyLock::new(|| SyntectAdapterBuilder::new().css().build());

static OPTIONS_SAFE: LazyLock<Options<'static>> = LazyLock::new(|| comrak_options(false));
static OPTIONS_UNSAFE: LazyLock<Options<'static>> = LazyLock::new(|| comrak_options(true));

fn comrak_options(allow_html: bool) -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.footnotes = true;
    options.extension.autolink = true;
    options.extension.math_dollars = true;
    // Empty prefix: `<h2 id="setup">`, the ids the outline jumps to.
    options.extension.header_id_prefix = Some(String::new());
    options.render.tasklist_classes = true;
    options.render.r#unsafe = allow_html;
    options
}

/// Renders Markdown to an HTML fragment, without the surrounding page.
pub fn render_fragment(markdown: &str, allow_html: bool) -> Rendered {
    let options = if allow_html {
        &*OPTIONS_UNSAFE
    } else {
        &*OPTIONS_SAFE
    };
    let adapter: &SyntectAdapter = &ADAPTER;
    let plugins = Plugins {
        render: RenderPlugins {
            codefence_syntax_highlighter: Some(adapter),
            ..Default::default()
        },
    };
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, options);
    let mut diagrams = Vec::new();
    let mut mermaid_blocks = 0;
    for node in root.descendants() {
        let raw = match &node.data.borrow().value {
            NodeValue::Math(NodeMath {
                literal,
                display_math,
                ..
            }) => crate::math::render(literal, *display_math),
            NodeValue::CodeBlock(block)
                if block.info.split_whitespace().next() == Some("chordpro") =>
            {
                crate::chords::render(&block.literal)
            }
            NodeValue::CodeBlock(block)
                if block.info.split_whitespace().next() == Some("mermaid")
                    && mermaid_blocks < crate::mermaid::MAX_DIAGRAMS =>
            {
                mermaid_blocks += 1;
                crate::mermaid::block(&block.literal).map(|diagram| {
                    if diagram.pending {
                        diagrams.push(block.literal.clone());
                    }
                    diagram.html
                })
            }
            _ => None,
        };
        // Raw is written unescaped even in safe mode; a formula or song that
        // doesn't parse, or a diagram without mmdc, keeps its node, so comrak shows its source.
        if let Some(raw) = raw {
            node.data.borrow_mut().value = NodeValue::Raw(raw);
        }
    }
    // Same slugs as in the HTML: comrak also starts a fresh Anchorizer per document.
    let mut anchorizer = Anchorizer::new();
    let headings = root
        .descendants()
        .filter_map(|node| match &node.data.borrow().value {
            NodeValue::Heading(heading) => {
                let text = node.collect_text();
                let id = anchorizer.anchorize(&text);
                Some(Heading {
                    level: heading.level,
                    text,
                    id,
                })
            }
            _ => None,
        })
        .collect();
    let mut html = String::new();
    // Writing into a String cannot fail; an error here would be a comrak bug.
    if format_html_with_plugins(root, options, &mut html, &plugins).is_err() {
        html.clear();
    }
    Rendered {
        html,
        headings,
        diagrams,
    }
}

/// Renders Markdown to a complete HTML page with the given CSS inlined.
pub fn render_page(markdown: &str, css: &str, opts: RenderOptions) -> Rendered {
    let img_src = if opts.allow_remote {
        "file: data: https:"
    } else {
        "file: data:"
    };
    let Rendered {
        html: body,
        headings,
        diagrams,
    } = render_fragment(markdown, opts.allow_html);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src {img_src}; style-src 'unsafe-inline'">
<style>
{css}
</style>
</head>
<body>
<main id="content">
{body}</main>
</body>
</html>
"#
    );
    Rendered {
        html,
        headings,
        diagrams,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_raw_html_by_default() {
        let html = render_fragment("<script>alert(1)</script>\n\n<b>bold</b>", false).html;
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn allow_html_passes_raw_html() {
        let html = render_fragment("<b>bold</b>", true).html;
        assert!(html.contains("<b>bold</b>"));
    }

    #[test]
    fn strips_javascript_links() {
        let html = render_fragment("[click](javascript:alert(1))", false).html;
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn highlights_code_with_classes_not_inline_styles() {
        let html = render_fragment("```rust\nfn main() {}\n```", false).html;
        assert!(html.contains("class=\""));
        assert!(!html.contains("style=\""));
    }

    #[test]
    fn renders_gfm_tables_and_tasklists() {
        let html = render_fragment("| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] done", false).html;
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("class=\"task-list-item\""));
    }

    #[test]
    fn page_has_csp_css_and_content() {
        let page = render_page("# Hi", "body { color: red; }", RenderOptions::default()).html;
        assert!(page.contains("default-src 'none'; img-src file: data:;"));
        assert!(page.contains("body { color: red; }"));
        assert!(page.contains("<main id=\"content\">\n<h1 id=\"hi\">"));
    }

    #[test]
    fn allow_remote_adds_https_to_csp() {
        let opts = RenderOptions {
            allow_remote: true,
            ..Default::default()
        };
        let page = render_page("", "", opts).html;
        assert!(page.contains("img-src file: data: https:;"));
    }

    #[test]
    fn math_becomes_inline_svg() {
        let html = render_fragment("Energy: $E = mc^2$ and\n\n$$\n\\frac{1}{2}\n$$\n", false).html;
        assert!(html.contains("<span class=\"math math-inline\"><svg "));
        assert!(html.contains("<span class=\"math math-display\"><svg "));
        assert!(!html.contains("data-math-style"));
    }

    #[test]
    fn broken_math_keeps_its_source() {
        let html = render_fragment("$\\frac{1}$", false).html;
        assert!(html.contains("<span data-math-style=\"inline\">\\frac{1}</span>"));
        assert!(!html.contains("<svg"));
    }

    #[test]
    fn chordpro_block_becomes_a_song() {
        let html = render_fragment(
            "```chordpro\n[Am]Hello\n```\n\n```text\n[Am]Hello\n```",
            false,
        )
        .html;
        assert_eq!(html.matches("<span class=\"chord\">Am</span>").count(), 1);
        assert!(html.contains("[Am]Hello"));
    }

    #[test]
    fn lone_dollars_stay_text() {
        let html = render_fragment("costs $5 and $10", false).html;
        assert!(html.contains("costs $5 and $10"));
        assert!(!html.contains("<svg"));
    }

    #[test]
    fn headings_get_unique_gfm_ids() {
        let rendered = render_fragment(
            "# Intro\n\n## Setup\n\n### **Bold** step\n\n## Setup",
            false,
        );
        let ids: Vec<(u8, &str, &str)> = rendered
            .headings
            .iter()
            .map(|h| (h.level, h.text.as_str(), h.id.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                (1, "Intro", "intro"),
                (2, "Setup", "setup"),
                (3, "Bold step", "bold-step"),
                (2, "Setup", "setup-1"),
            ]
        );
        assert!(rendered.html.contains("<h3 id=\"bold-step\">"));
        assert!(rendered.html.contains("<h2 id=\"setup-1\">"));
    }
}
