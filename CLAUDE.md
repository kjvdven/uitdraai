# Working agreements for uitdraai

The plan and the phases live in `PLAN.md`. This file describes how we build.

## Way of working
- Work on one phase from `PLAN.md` at a time and stop afterwards for review.
- Tick off finished tasks in `PLAN.md`.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` must pass before you report a step as done.
- Small, logical commits with a clear message per step.
- In doubt about a design choice or an open question from `PLAN.md`: ask, don't guess.

## Simplicity (beats DRY)
- No trait or abstraction for something with only one implementation.
- No new crate without proposing it first and explaining why.
- Duplication is fine until it happens the third time; only then merge.
- Prefer a slightly longer function that reads well over three small ones you have to follow around.

## Single source of truth
- There is exactly one render path (`render.rs`). Preview and export both use it.
- Themes and defaults live in one place; no hardcoded CSS in Rust code.
- All WebKit code stays inside `src/gui/`. `render.rs`, `export.rs` and `watch.rs` must know nothing about WebKit or GTK.

## Security
- Treat every `.md` file as untrusted input.
- No JavaScript in the rendered page. Highlighting (and later diagrams and formulas) is rendered up front in Rust. The only JS is the live reload the app itself runs via `evaluate_javascript`.
- comrak: raw HTML off by default (`render.unsafe_ = false`). Only on via the explicit `--allow-html` flag.
- WebView settings:
  - `enable-javascript-markup = false` (`evaluate_javascript` keeps working for the live reload)
  - `allow-file-access-from-file-urls = false`
  - `allow-universal-access-from-file-urls = false`
- `decide-policy`: only allow the initial load; external links go to the system browser, block all other navigation.
- HTML passed through `evaluate_javascript` is always escaped with `serde_json::to_string`, never via string concatenation.
- Subprocesses (pandoc, weasyprint) always via `std::process::Command` with separate args, never via a shell. Put `--` before file names, so a file starting with a dash isn't read as an option.
- No network traffic while rendering, previewing or exporting. Remote content only via the explicit `--allow-remote` flag.
- The rendered page gets a CSP meta tag: `default-src 'none'; img-src file: data:; style-src 'unsafe-inline'`. With `--allow-remote`, `https:` is added to `img-src`. That blocks scripts and remote content even if another layer fails.
- No `unwrap()` or `expect()` outside tests; errors via `anyhow` with `.context(...)`.
- Run `cargo audit` before adding a dependency.

## Performance
Targets (release build, document of ~1000 lines):
- Window visible with content: < 300 ms after start.
- Preview updated after save: < 50 ms (excluding debounce).
- Measure this with the `--timing` flag, which logs the duration of each step to stderr.

Rules:
- Only optimise what has been measured to be slow, except for the points below.
- Build the syntect `SyntaxSet` and the comrak options once (`std::sync::LazyLock`), never per render.
- Highlighting with CSS classes (`ClassedHTMLGenerator`), no inline styles. That gives smaller HTML, and switching themes works without re-rendering.
- Rendering happens off the GTK main thread; only the result goes to the UI via a channel.
- On live reload, only replace the contents of `#content`, never reload the whole page.

Release profile in `Cargo.toml`:

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

## Rust practices
- Edition 2024. `Cargo.lock` is committed (it's a binary).
- Public functions get a short doc comment; no comments that just repeat what the code already says.
- Unit tests next to the code (`mod tests`) for logic; integration tests in `tests/` for the CLI.
- Tests that need external tools (pandoc, weasyprint) are skipped when those tools are missing, instead of failing.

## Releases
- The version lives in three places: `Cargo.toml`, `Cargo.lock` (via `cargo build`) and `flake.nix`. Bump all three in one `chore: bump version to X.Y.Z` PR.
- After the merge: `gh release create vX.Y.Z --target main` with one bullet per merged feature since the previous tag. The tag is created by the release.
- Semver, pre-1.0: a new feature bumps minor, a fix bumps patch.
