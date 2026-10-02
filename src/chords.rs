use std::sync::LazyLock;

use chordsketch_chordpro::config::Config;

// chordsketch runs abc2svg, lilypond or musescore when they are installed.
// A song can't turn them back on: `{+config.delegates...}` is not on its allowlist.
static CONFIG: LazyLock<Option<Config>> = LazyLock::new(|| {
    Config::parse(r#"{"delegates": {"abc2svg": false, "lilypond": false, "musescore": false}}"#)
        .ok()
        .map(|delegates_off| Config::defaults().merge(delegates_off))
});

/// Renders a ChordPro song to HTML with the chords above the lyrics.
/// Returns `None` when it doesn't parse; the caller shows the source.
pub fn render(source: &str) -> Option<String> {
    let config = CONFIG.as_ref()?;
    let song = chordsketch_chordpro::parse(source).ok()?;
    // Warnings are about the song (unknown key, odd capo); it still renders.
    Some(chordsketch_render_html::render_song_body_with_warnings(&song, 0, config).output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chords_sit_above_their_lyrics() {
        let html = render("{title: Song}\n[Am]Hello [G]world").unwrap();
        assert!(html.contains("<h1>Song</h1>"));
        assert!(
            html.contains("<span class=\"chord\">Am</span><span class=\"lyrics\">Hello </span>")
        );
    }

    #[test]
    fn chordsize_sets_the_font_size() {
        let html = render("{chordsize: 14}\n[Am]Hello").unwrap();
        assert!(html.contains("font-size: 14pt;"));
    }

    #[test]
    fn markup_in_the_song_is_escaped() {
        let html =
            render("{title: <img src=x onerror=alert(1)>}\n[<b>C</b>]<script>x</script>").unwrap();
        assert!(!html.contains("<img"));
        assert!(!html.contains("<b>"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn svg_sections_lose_scripts_and_external_links() {
        let html = render(
            "{start_of_svg}\n<svg onload=\"x\"><script>x</script><use href=\"https://evil.example/a#b\"/></svg>\n{end_of_svg}",
        )
        .unwrap();
        assert!(!html.contains("onload"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("evil.example"));
    }

    #[test]
    fn abc_stays_text_even_when_a_song_asks_for_the_tool() {
        let html = render(
            "{+config.delegates.abc2svg: true}\n{start_of_abc}\nX:1\nK:C\nCDEC|\n{end_of_abc}",
        )
        .unwrap();
        assert!(html.contains("<span class=\"lyrics\">CDEC|</span>"));
        assert!(!html.contains("<svg"));
    }
}
