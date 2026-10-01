# uitdraai: plan

## Goal
A lightweight Markdown previewer for Linux, Wayland-first (niri, Hyprland), written in Rust. It shows one file live with your own CSS and exports to PDF (DOCX/ODT follows in phase 3). The CLI is the foundation. The GUI is a thin GTK4 window on top of the same core.

Working agreements (simplicity, security, performance, Rust practices) live in `CLAUDE.md`.

## Out of scope (v1)
No editor, no tabs or multiple files, no plugins, no scroll sync with an editor, no X11-specific workarounds and no automatic light/dark mode (white by default; a dark variant is a theme of its own). Mermaid and KaTeX come in phase 3.

## Architecture
One crate with a `gui` feature, so the CLI also builds without WebKit dependencies.

```
uitdraai/
├── Cargo.toml          # features: default = ["gui"], gui = ["webkit6", "gtk4", "async-channel", "serde_json"]
├── CLAUDE.md
├── PLAN.md
├── themes/             # built-in themes, embedded via include_str!
│   ├── default.css
│   └── print.css       # @page, margins, page numbers
└── src/
    ├── main.rs         # clap: subcommands and dispatch
    ├── render.rs       # markdown → HTML fragment and complete page
    ├── theme.rs        # theme lookup: --css, config dir, built-in
    ├── export.rs       # PDF (weasyprint); DOCX/ODT via pandoc in phase 3
    ├── watch.rs        # file watching with debounce
    ├── config.rs       # ~/.config/uitdraai/config.toml
    └── gui/            # only with feature "gui"
        └── mod.rs
```

### Crates
- `comrak`: Markdown with GFM (tables, task lists, footnotes, strikethrough).
- `syntect`: code highlighting, hooked into comrak via the syntect adapter, with CSS classes.
- `clap` (derive): CLI.
- `notify-debouncer-mini`: file watching (re-exports `notify` itself; the rename tracking of `-full` isn't needed, since we watch the directory and filter by name).
- `serde`, `serde_json`, `toml`: configuration and safe escaping towards JavaScript.
- `directories`: config paths following XDG.
- `anyhow`: error handling.
- `gtk4` + `webkit6`: GUI (only with feature `gui`).
- `async-channel`: results from the render thread to the GTK main thread (only with feature `gui`; replaces the removed `glib::MainContext::channel`).

New crates only after discussion (see `CLAUDE.md`).

### Design decisions
**One render path.** `render.rs` produces a complete HTML page with the CSS inlined. Preview and PDF export get exactly the same HTML and CSS. Note: they are drawn by two different engines (WebKitGTK for the preview, WeasyPrint for the PDF), so small differences are possible. Keep themes to document typography and avoid complex layout (grid, advanced flexbox).

**No JavaScript in the page.** The preview needs no JS. Everything that looks dynamic (highlighting, later diagrams and formulas) is rendered up front in Rust. Only live reload uses `evaluate_javascript`, from the app itself.

**Viewer separate from the rest.** All WebKit code stays inside `src/gui/`. `render.rs`, `export.rs` and `watch.rs` know nothing about WebKit. That way the viewer can be replaced later (see Blitz in phase 3) without touching the rest. No trait or abstraction layer for it now; only once a second viewer actually arrives.

**Raw HTML off by default.** Markdown is treated as untrusted input. Raw HTML in Markdown is not passed through unless the user gives `--allow-html`. Remote content is blocked by default, among other things via a CSP meta tag in the page; `--allow-remote` turns it on.

**Relative images must work.** The GUI loads the HTML with `load_html(html, Some("file:///path/to/dir/"))`, and WeasyPrint gets `--base-url`.

**External tools for export.** PDF goes through the `weasyprint` binary, for the best support of print CSS (`@page`, `counter(page)`, page breaks). If WeasyPrint is missing, the user gets a clear error with install instructions.

**DOCX/ODT in phase 3.** Rarely used, and pandoc reads the Markdown directly, bypassing `render.rs`. It's a deliberate exception to "one render path": no CSS, Word styling via `--reference-doc`. Without `--allow-html` it becomes `pandoc -f gfm-raw_html`.

**Watch the directory, not the file.** Many editors (Vim, Helix) save atomically via rename, so a watch on the file itself goes silent after the first save. Watch the parent directory instead, filter by file name and debounce by about 100 ms.

**Live reload without flicker.** On a change, the page isn't reloaded; only the contents of `#content` are replaced via `evaluate_javascript`. The HTML is escaped with `serde_json::to_string`. That keeps the scroll position for free. A theme switch likewise only replaces the contents of `<style>` and keeps the scroll position as a ratio. Only `Ctrl+R` reloads the whole page.

## CLI specification
```
uitdraai <file.md>                         # opens the GUI (with feature gui)
uitdraai render <file.md> [-o out.html]    # HTML to a file or stdout
uitdraai export <file.md> --pdf [-o <dir>]
uitdraai export <file.md> --pdf --watch    # re-export on every save
uitdraai themes                            # list available themes
uitdraai themes --dump <name>              # theme to stdout, as a base for your own

Global options:
  --css <path>       own stylesheet (wins over --theme)
  --theme <name>     theme from the config dir or built in
  --allow-html       allow raw HTML in Markdown
  --allow-remote     load remote images (blocked by default)
  --timing           log the duration of each step to stderr
```

Theme lookup order: `--css`, then `~/.config/uitdraai/themes/<name>.css`, then the built-in themes. `print.css` is always loaded before the theme, so an `@media print` block in your own theme can override the print rules. You make your own theme by dumping an existing one and editing it; themes don't stack.

## GUI specification
- App id `io.github.kjvdven.uitdraai`, so window rules can match it.
- Minimal toolbar at the top with a headings dropdown (jump to a heading, `Ctrl+Shift+T` or `F9`), a theme dropdown, "Open in editor" (default app for Markdown, `Ctrl+Shift+O`) and one export split button: the main part exports the default format (PDF, or `default_export = "html"` in `config.toml`), the arrow offers PDF and HTML. Toolbar and status bar can be hidden together with `--no-toolbar` or `Ctrl+Shift+H` (or `F11`), because on a tiling WM you often only want the content.
- Shortcuts: `Ctrl+O` open (in a new window), `Ctrl+E` export in the default format (save dialog, name and folder prefilled), `Ctrl+Shift+E` choose the export format (keyboard-navigable menu), `Ctrl+R` full reload, `Ctrl+Shift+T` / `F9` headings dropdown, `Ctrl+Shift+H` / `F11` toolbar and status bar, `Ctrl+,` open config.toml (created from a template first), `Ctrl+?` overview of all shortcuts, `Ctrl+Q` close, `Ctrl++` / `Ctrl+-` / `Ctrl+0` zoom via `WebView::set_zoom_level`.
- Status bar at the bottom with the last event (opened, updated with time and duration, export, errors), no popups. After an export, "Open PDF"/"Open HTML" and "Show in folder" buttons for the last exported file.
- External links open in the default browser; all other navigation is blocked (via the `decide-policy` signal).

Example window rules:

```kdl
// niri
window-rule {
    match app-id="^io\\.github\\.kjvdven\\.uitdraai$"
    default-column-width { proportion 0.4; }
}
```

```
# Hyprland
windowrulev2 = float, class:^(io\.github\.kjvdven\.uitdraai)$
windowrulev2 = size 40% 90%, class:^(io\.github\.kjvdven\.uitdraai)$
```

## Phases

### Phase 1: core + CLI
Rendering and exporting work from the terminal.

- [x] Cargo project with a `gui` feature (empty for now) and the release profile from `CLAUDE.md`
- [x] `render.rs`: comrak (GFM), syntect with CSS classes, raw HTML off by default
- [x] Built-in themes `default.css` and `print.css`, including highlighting CSS
- [x] `theme.rs` with the lookup order
- [x] `main.rs`: CLI with clap (derive), subcommands `render` and `themes [--dump]` and the global options; `export` follows with `export.rs`
- [x] `export.rs`: PDF via weasyprint, with tool detection
- [x] `watch.rs` with directory watch and debounce, plus `export --watch`
- [x] `--timing` flag
- [x] Tests for rendering and theme resolution; a smoke test for export that is skipped when the tools are missing
- [x] Fixture `tests/fixtures/large.md` of ~1000 lines (headings, tables, code blocks) as a fixed benchmark document. No timing asserts in `cargo test`; they are flaky

**Done when:** `uitdraai export notes.md --pdf` produces a correct PDF, relative images show up in the PDF, a `<script>` in the Markdown file doesn't end up in the output, and `uitdraai render tests/fixtures/large.md --timing` (release build) shows how long rendering takes, as a baseline for phase 2.

### Phase 2: GUI
A live preview window on Wayland.

- [x] GTK4 application with app id, WebView and `load_html` with a base URI
- [x] WebView settings locked down as described in `CLAUDE.md`
- [x] Watcher and rendering in their own thread, result via `async-channel` to the UI, received with `glib::spawn_future_local`
- [x] Replace the content via JS so a reload doesn't jump
- [x] Toolbar with theme picker and PDF export button, plus shortcuts
- [x] Navigation policy: external links to the browser, block the rest; context menu off (reload and back would navigate away from the page)

**Done when:** you open a file in niri or Hyprland, save it in your editor, and the preview updates without jumping, and `--timing` (release build, `tests/fixtures/large.md`) meets the performance targets from `CLAUDE.md`: window with content < 300 ms, update after save < 50 ms.

### Phase 3: extras
- [ ] Render Mermaid and KaTeX to SVG in Rust up front, without JavaScript in the page. Mermaid research and options: [#1](https://github.com/kjvdven/uitdraai/issues/1); parked to keep the app simple. Math research and options: [#16](https://github.com/kjvdven/uitdraai/issues/16)
- [ ] Reconsider Blitz as a lighter viewer (pure Rust, no WebKit, no web process). Only worthwhile once the CSS the themes use is well supported there; compare Blitz's roadmap with the themes first
- [ ] DOCX/ODT export via pandoc (`--docx`, `--odt`, `--reference-doc`), see the design decision "DOCX/ODT in phase 3". Look into pandoc's `--sandbox` for remote content. Math: add `+tex_math_dollars` to the `gfm` reader, pandoc then writes editable Word equations (OMML) and MathML for ODT. Mermaid: pandoc has no support; optional `mmdc` (mermaid-cli) as a subprocess that renders the blocks to PNG before pandoc runs, same pattern as weasyprint. Without it the block stays a code block
- [x] Headings dropdown in the toolbar (`Ctrl+Shift+T`, `F9`); headings get GFM ids
- [x] `config.toml` with `theme`, `export_dir`, `editor` (a list, e.g. `["ghostty", "-e", "hx"]`, so terminal editors work too) `default_export` (`"pdf"` or `"html"`) and `disable_dmabuf` (NVIDIA). The latter sets `WEBKIT_DISABLE_DMABUF_RENDERER` in `gui::prepare` before GTK starts and before any thread, with a `// SAFETY:` comment. `reference-doc` follows with DOCX

## Dependencies per distro
- **Arch:** `gtk4 webkitgtk-6.0 pandoc python-weasyprint`
- **Fedora:** `gtk4-devel webkitgtk6.0-devel pandoc weasyprint`
- **Debian/Ubuntu 24.04+:** `libgtk-4-dev libwebkitgtk-6.0-dev pandoc weasyprint`
- **NixOS:** `nix develop` (see `flake.nix`) provides the libraries and weasyprint; Rust comes from `mise.toml`.

## Risks and open questions
- **NVIDIA:** WebKitGTK can show an empty window. Workaround: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, available as `disable_dmabuf = true` in `config.toml`.
- **WeasyPrint needs Python on the system.** Decided: we accept this, because WeasyPrint has the best print CSS support and Rust has no mature alternative of its own. WebKit's `PrintOperation` and Typst were considered and rejected (limited print CSS, and no CSS styling, respectively).
- **Preview and PDF use different engines.** See the design decision "One render path". If differences become a nuisance in practice, an optional `--pdf-engine webkit` could be added.
- **WebKitGTK is big.** Over 100 MB on disk and roughly 80 to 150 MB RAM per window. Accepted for now; Blitz is the alternative for later (phase 3). If you only want the CLI, build without the `gui` feature.
- **Syntect cold start.** Phase 1 baseline (release, `tests/fixtures/large.md`): first render ~180 ms, after that ~16 ms, without code blocks ~1 ms. Almost all of it is one-time syntect initialisation. Fits the 300 ms startup target, but only just. Options if it turns out too slow: warm up the `LazyLock` in a thread in parallel with GTK init, or `syntect-onig` instead of `syntect-fancy`.
- **Mermaid and KaTeX without JavaScript.** Mermaid is researched (see #1): `merman` or `mmdflux` can render SVG in pure Rust; Math is researched (see #16): `pulldown-latex` renders LaTeX to MathML in pure Rust, and WebKitGTK shows MathML natively; open point is the PDF export, since weasyprint has no MathML. If that doesn't work well, the fallback is our own embedded scripts via `UserContentManager` (never scripts from the Markdown file). Decide at the start of phase 3.
- **Remote content.** Decided: off by default, only on via `--allow-remote`. In the preview the CSP meta tag blocks it (see `CLAUDE.md`). WeasyPrint gets `--allowed-protocols file,data` (with `--allow-remote` also `https`); the CSP meta tag doesn't apply there.
