use std::sync::LazyLock;

use comrak::options::{Plugins, RenderPlugins};
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Options, markdown_to_html_with_plugins};

/// Switches that loosen the default security settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderOptions {
    /// Pass raw HTML in the Markdown through (`--allow-html`).
    pub allow_html: bool,
    /// Allow remote images in the page (`--allow-remote`).
    pub allow_remote: bool,
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
    options.render.r#unsafe = allow_html;
    options
}

/// Renders Markdown to an HTML fragment, without the surrounding page.
pub fn render_fragment(markdown: &str, allow_html: bool) -> String {
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
    markdown_to_html_with_plugins(markdown, options, &plugins)
}

/// Renders Markdown to a complete HTML page with the given CSS inlined.
pub fn render_page(markdown: &str, css: &str, opts: RenderOptions) -> String {
    let img_src = if opts.allow_remote {
        "file: data: https:"
    } else {
        "file: data:"
    };
    let body = render_fragment(markdown, opts.allow_html);
    format!(
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_raw_html_by_default() {
        let html = render_fragment("<script>alert(1)</script>\n\n<b>bold</b>", false);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn allow_html_passes_raw_html() {
        let html = render_fragment("<b>bold</b>", true);
        assert!(html.contains("<b>bold</b>"));
    }

    #[test]
    fn strips_javascript_links() {
        let html = render_fragment("[click](javascript:alert(1))", false);
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn highlights_code_with_classes_not_inline_styles() {
        let html = render_fragment("```rust\nfn main() {}\n```", false);
        assert!(html.contains("class=\""));
        assert!(!html.contains("style=\""));
    }

    #[test]
    fn renders_gfm_tables_and_tasklists() {
        let html = render_fragment("| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] done", false);
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
    }

    #[test]
    fn page_has_csp_css_and_content() {
        let page = render_page("# Hi", "body { color: red; }", RenderOptions::default());
        assert!(page.contains("default-src 'none'; img-src file: data:;"));
        assert!(page.contains("body { color: red; }"));
        assert!(page.contains("<main id=\"content\">\n<h1>Hi</h1>"));
    }

    #[test]
    fn allow_remote_adds_https_to_csp() {
        let opts = RenderOptions {
            allow_remote: true,
            ..Default::default()
        };
        let page = render_page("", "", opts);
        assert!(page.contains("img-src file: data: https:;"));
    }
}
