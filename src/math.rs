use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parser::parse;
use ratex_svg::{SvgOptions, render_to_svg};
use ratex_types::math_style::MathStyle;

// User units per em in the SVG; the width and height attributes are rewritten
// to em so the formula scales with the surrounding text.
const FONT_SIZE: f64 = 16.0;

/// Renders a LaTeX formula to an inline SVG wrapped in a `.math` element.
/// Returns `None` when the formula doesn't parse; the caller shows the source.
pub fn render(latex: &str, display: bool) -> Option<String> {
    let style = if display {
        MathStyle::Display
    } else {
        MathStyle::Text
    };
    let ast = parse(latex).ok()?;
    let list = to_display_list(&layout(&ast, &LayoutOptions::default().with_style(style)));
    let opts = SvgOptions {
        font_size: FONT_SIZE,
        padding: 0.0,
        embed_glyphs: true,
        ..SvgOptions::default()
    };
    let svg = render_to_svg(&list, &opts);
    if looks_unsafe(&svg) {
        return None;
    }
    let view_box = attribute(&svg, "viewBox")?;
    let body = svg.get(svg.find('>')? + 1..)?;
    let width = list.width;
    let height = list.height + list.depth;
    let depth = list.depth;
    let body = body
        .replace("fill=\"rgba(0,0,0,1)\"", "fill=\"currentColor\"")
        .replace("stroke=\"rgba(0,0,0,1)\"", "stroke=\"currentColor\"");
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{view_box}\" width=\"{width:.3}em\" height=\"{height:.3}em\" style=\"vertical-align:-{depth:.3}em\">{body}"
    );
    // A span for both: the display form sits inside a `<p>`, where a div is invalid.
    let class = if display {
        "math-display"
    } else {
        "math-inline"
    };
    Some(format!("<span class=\"math {class}\">{svg}</span>"))
}

/// The glyphs are paths, so nothing but drawing elements belongs in the SVG.
fn looks_unsafe(svg: &str) -> bool {
    let lower = svg.to_ascii_lowercase();
    ["<script", "<a ", "href=", "javascript:", " on"]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_formula_scales_with_text_and_sits_on_the_baseline() {
        let html = render(r"\frac{1}{2}", false).unwrap();
        assert!(html.starts_with("<span class=\"math math-inline\"><svg "));
        assert!(html.contains("em\" height=\""));
        assert!(html.contains("style=\"vertical-align:-0."));
        assert!(html.contains("fill=\"currentColor\""));
        assert!(!html.contains("rgba(0,0,0,1)"));
    }

    #[test]
    fn display_formula_is_a_block() {
        let html = render(r"x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}", true).unwrap();
        assert!(html.starts_with("<span class=\"math math-display\">"));
        assert!(html.ends_with("</svg></span>"));
    }

    #[test]
    fn links_and_markup_never_reach_the_svg() {
        for latex in [
            r"\href{javascript:alert(1)}{x}",
            r"\url{javascript:alert(1)}",
            r"\text{<script>alert(1)</script>}",
        ] {
            let html = render(latex, false).unwrap();
            assert!(!looks_unsafe(&html), "{latex}");
            assert!(!html.contains("<script"), "{latex}");
        }
    }

    #[test]
    fn hostile_input_is_rejected_not_rendered() {
        assert!(render(r#"\color{red" onload="alert(1)} x"#, false).is_none());
        assert!(render(&"{".repeat(5000), false).is_none());
        let bomb = r"\def\a{xxxxxxxxxx}\def\b{\a\a\a\a\a\a\a\a\a\a}\def\c{\b\b\b\b\b\b\b\b\b\b}\def\d{\c\c\c\c\c\c\c\c\c\c}\def\e{\d\d\d\d\d\d\d\d\d\d}\e";
        assert!(render(bomb, false).is_none());
    }

    #[test]
    fn unbalanced_input_returns_none() {
        assert!(render(r"\frac{1}", false).is_none());
    }
}
