mod export;
#[cfg(feature = "gui")]
mod gui;
mod render;
mod theme;
mod watch;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::render::RenderOptions;

/// Lightweight Markdown previewer with PDF export.
#[derive(Parser)]
#[command(
    version,
    args_conflicts_with_subcommands = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Markdown file to open in the preview window
    file: Option<PathBuf>,

    #[command(flatten)]
    global: GlobalOpts,
}

#[derive(Args)]
struct GlobalOpts {
    /// Own stylesheet (wins over --theme)
    #[arg(long, global = true, value_name = "PATH")]
    css: Option<PathBuf>,

    /// Theme from the config dir or built in
    #[arg(long, global = true, value_name = "NAME")]
    theme: Option<String>,

    /// Allow raw HTML in the Markdown
    #[arg(long, global = true)]
    allow_html: bool,

    /// Load remote images (blocked by default)
    #[arg(long, global = true)]
    allow_remote: bool,

    /// Log the duration of each step to stderr
    #[arg(long, global = true)]
    timing: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Render Markdown to a complete HTML page
    Render {
        file: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Export Markdown to PDF
    Export {
        file: PathBuf,

        /// Export to PDF (via weasyprint)
        #[arg(long, required = true)]
        pdf: bool,

        /// Output directory (default: next to the Markdown file)
        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,

        /// Export again after every save
        #[arg(long)]
        watch: bool,
    },
    /// List available themes
    Themes {
        /// Print a theme's CSS, as a starting point for your own
        #[arg(long, value_name = "NAME")]
        dump: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme_dir = theme::user_theme_dir();
    match cli.command {
        Some(Command::Render { file, output }) => {
            let html = render_file(&file, &cli.global, theme_dir.as_deref())?;
            match output {
                Some(path) => fs::write(&path, html)
                    .with_context(|| format!("cannot write {}", path.display()))?,
                None => io::stdout()
                    .write_all(html.as_bytes())
                    .context("cannot write to stdout")?,
            }
        }
        Some(Command::Export {
            file,
            pdf: _,
            output,
            watch,
        }) => {
            let export =
                || export_file(&file, output.as_deref(), &cli.global, theme_dir.as_deref());
            export()?;
            if watch {
                eprintln!("watching {}, Ctrl+C to stop", file.display());
                watch::watch(&file, || {
                    if let Err(err) = export() {
                        eprintln!("Error: {err:#}");
                    }
                })?;
            }
        }
        Some(Command::Themes { dump: Some(name) }) => {
            print!("{}", theme::load(&name, theme_dir.as_deref())?);
        }
        Some(Command::Themes { dump: None }) => {
            for name in theme::list(theme_dir.as_deref())? {
                println!("{name}");
            }
        }
        None => {
            let file = cli.file.context("no Markdown file given")?;
            open_preview(&file, &cli.global, theme_dir.as_deref())?;
        }
    }
    Ok(())
}

#[cfg(feature = "gui")]
fn open_preview(file: &Path, global: &GlobalOpts, theme_dir: Option<&Path>) -> Result<()> {
    let page = render_file(file, global, theme_dir)?;
    let file =
        fs::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let (watched, timing, allow_html) = (file.clone(), global.timing, global.allow_html);
    gui::run(page, file, move || {
        let markdown = timed(timing, "read", || {
            fs::read_to_string(&watched)
                .with_context(|| format!("cannot read {}", watched.display()))
        })?;
        Ok(timed(timing, "render", || {
            render::render_fragment(&markdown, allow_html)
        }))
    })
}

#[cfg(not(feature = "gui"))]
fn open_preview(_file: &Path, _global: &GlobalOpts, _theme_dir: Option<&Path>) -> Result<()> {
    anyhow::bail!("built without the preview window; use `uitdraai render` or `uitdraai export`")
}

fn export_file(
    file: &Path,
    output: Option<&Path>,
    global: &GlobalOpts,
    theme_dir: Option<&Path>,
) -> Result<()> {
    let html = render_file(file, global, theme_dir)?;
    let file =
        fs::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let base_dir = file
        .parent()
        .context("Markdown file has no parent directory")?;
    let name = file.with_extension("pdf");
    let pdf = output
        .unwrap_or(base_dir)
        .join(name.file_name().context("no file name")?);
    timed(global.timing, "export", || {
        export::export_pdf(&html, base_dir, &pdf, global.allow_remote)
    })?;
    eprintln!("wrote {}", pdf.display());
    Ok(())
}

fn render_file(file: &Path, global: &GlobalOpts, theme_dir: Option<&Path>) -> Result<String> {
    let markdown = timed(global.timing, "read", || {
        fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))
    })?;
    let css = timed(global.timing, "theme", || {
        theme::resolve(global.css.as_deref(), global.theme.as_deref(), theme_dir)
    })?;
    let opts = RenderOptions {
        allow_html: global.allow_html,
        allow_remote: global.allow_remote,
    };
    Ok(timed(global.timing, "render", || {
        render::render_page(&markdown, &css, opts)
    }))
}

/// Runs `step` and, with `--timing`, logs how long it took to stderr.
fn timed<T>(enabled: bool, label: &str, step: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = step();
    if enabled {
        eprintln!(
            "timing: {label} {:.1} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    result
}
