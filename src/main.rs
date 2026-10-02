mod chords;
mod config;
mod export;
#[cfg(feature = "gui")]
mod gui;
mod math;
mod render;
mod theme;
mod watch;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

use crate::render::{RenderOptions, Rendered};

/// Lightweight Markdown previewer with PDF export.
#[derive(Parser)]
#[command(version, args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Markdown file to open in the preview window
    file: Option<PathBuf>,

    /// Start the preview window without toolbar and status bar (Ctrl+Shift+H shows them)
    #[arg(long)]
    no_toolbar: bool,

    #[command(flatten)]
    global: GlobalOpts,
}

#[derive(Args, Clone)]
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
    let mut cli = Cli::parse();
    let config_path = config::path();
    let config = config::load(config_path.as_deref())?;
    if cli.global.theme.is_none() {
        cli.global.theme.clone_from(&config.theme);
    }
    let theme_dir = theme::user_theme_dir();
    match cli.command {
        Some(Command::Render { file, output }) => {
            let html = render_file(&file, &cli.global, theme_dir.as_deref())?.html;
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
            let export = || -> Result<()> {
                let dir = output.as_deref().or(config.export_dir.as_deref());
                let pdf = default_pdf_path(&file, dir)?;
                export_file(&file, &pdf, &cli.global, theme_dir.as_deref())?;
                eprintln!("wrote {}", pdf.display());
                Ok(())
            };
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
            let preview = Preview {
                show_bars: !cli.no_toolbar,
                config,
                config_path,
            };
            open_preview(cli.file, &cli.global, theme_dir.as_deref(), preview)?;
        }
    }
    Ok(())
}

/// Window settings that come from outside the global flags.
#[cfg_attr(
    not(feature = "gui"),
    allow(dead_code, reason = "only the preview window uses it")
)]
struct Preview {
    show_bars: bool,
    config: config::Config,
    config_path: Option<PathBuf>,
}

/// Opens the preview window; without a file it asks for one first (launchers pass none).
#[cfg(feature = "gui")]
fn open_preview(
    file: Option<PathBuf>,
    global: &GlobalOpts,
    theme_dir: Option<&Path>,
    preview: Preview,
) -> Result<()> {
    gui::prepare(preview.config.disable_dmabuf);
    let file = match file {
        Some(file) => file,
        None => match gui::pick_file()? {
            Some(file) => file,
            None => return Ok(()),
        },
    };
    let file = file.as_path();
    let page = render_file(file, global, theme_dir)?;
    let file =
        fs::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let render_page = {
        let (file, global, theme_dir) =
            (file.clone(), global.clone(), theme_dir.map(Path::to_owned));
        move |theme: Option<&str>| {
            render_file(&file, &with_theme(&global, theme), theme_dir.as_deref())
        }
    };
    let theme_css = {
        let (global, theme_dir) = (global.clone(), theme_dir.map(Path::to_owned));
        move |theme: Option<&str>| {
            let global = with_theme(&global, theme);
            theme::resolve(
                global.css.as_deref(),
                global.theme.as_deref(),
                theme_dir.as_deref(),
            )
        }
    };
    let render_content = {
        let (file, timing, allow_html) = (file.clone(), global.timing, global.allow_html);
        move || {
            let markdown = timed(timing, "read", || {
                fs::read_to_string(&file).with_context(|| format!("cannot read {}", file.display()))
            })?;
            Ok(timed(timing, "render", || {
                render::render_fragment(&markdown, allow_html)
            }))
        }
    };
    let export_pdf = {
        let (file, global, theme_dir) =
            (file.clone(), global.clone(), theme_dir.map(Path::to_owned));
        move |pdf: &Path, theme: Option<&str>| {
            export_file(
                &file,
                pdf,
                &with_theme(&global, theme),
                theme_dir.as_deref(),
            )
        }
    };
    let export_html = {
        let (file, global, theme_dir) =
            (file.clone(), global.clone(), theme_dir.map(Path::to_owned));
        move |html_file: &Path, theme: Option<&str>| {
            let html = render_file(&file, &with_theme(&global, theme), theme_dir.as_deref())?.html;
            fs::write(html_file, html)
                .with_context(|| format!("cannot write {}", html_file.display()))
        }
    };
    let open = {
        let global = global.clone();
        move |file: &Path| spawn_preview(file, &global)
    };
    let themes = theme::list(theme_dir)?;
    // Built-in themes are compiled in, so only `--css` and user themes are files to watch.
    let mut stylesheets: Vec<PathBuf> = global.css.iter().cloned().collect();
    if let Some(dir) = theme_dir {
        stylesheets.extend(
            themes
                .iter()
                .map(|name| dir.join(format!("{name}.css")))
                .filter(|path| path.is_file()),
        );
    }
    let options = gui::Options {
        page,
        themes,
        theme: match global.css {
            Some(_) => None,
            None => Some(global.theme.clone().unwrap_or_else(|| "default".to_owned())),
        },
        stylesheets,
        default_pdf: default_pdf_path(&file, preview.config.export_dir.as_deref())?,
        editor: preview.config.editor,
        default_export: preview.config.default_export,
        config_path: preview.config_path,
        show_bars: preview.show_bars,
    };
    gui::run(
        file,
        options,
        gui::Hooks {
            render_page: Box::new(render_page),
            theme_css: Box::new(theme_css),
            render_content: Box::new(render_content),
            export_pdf: Box::new(export_pdf),
            export_html: Box::new(export_html),
            open: Box::new(open),
        },
    )
}

/// The options with `theme` picked in the window, which replaces `--css` and `--theme`.
#[cfg(feature = "gui")]
fn with_theme(global: &GlobalOpts, theme: Option<&str>) -> GlobalOpts {
    let mut global = global.clone();
    if let Some(theme) = theme {
        global.css = None;
        global.theme = Some(theme.to_owned());
    }
    global
}

#[cfg(not(feature = "gui"))]
fn open_preview(
    _file: Option<PathBuf>,
    _global: &GlobalOpts,
    _theme_dir: Option<&Path>,
    _preview: Preview,
) -> Result<()> {
    anyhow::bail!("built without the preview window; use `uitdraai render` or `uitdraai export`")
}

/// Where `export` writes by default: `<name>.pdf` in `output`, or next to the Markdown file.
fn default_pdf_path(file: &Path, output: Option<&Path>) -> Result<PathBuf> {
    let file =
        fs::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let dir = match output {
        Some(dir) => dir,
        None => file
            .parent()
            .context("Markdown file has no parent directory")?,
    };
    let name = file.with_extension("pdf");
    Ok(dir.join(name.file_name().context("no file name")?))
}

fn export_file(
    file: &Path,
    pdf: &Path,
    global: &GlobalOpts,
    theme_dir: Option<&Path>,
) -> Result<()> {
    let html = render_file(file, global, theme_dir)?.html;
    let file =
        fs::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let base_dir = file
        .parent()
        .context("Markdown file has no parent directory")?;
    timed(global.timing, "export", || {
        export::export_pdf(&html, base_dir, pdf, global.allow_remote)
    })
}

fn render_file(file: &Path, global: &GlobalOpts, theme_dir: Option<&Path>) -> Result<Rendered> {
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

/// Opens `file` in a new preview window with the same options, as a separate process.
#[cfg(feature = "gui")]
fn spawn_preview(file: &Path, global: &GlobalOpts) -> Result<()> {
    let exe = std::env::current_exe().context("cannot find the uitdraai binary")?;
    let mut command = std::process::Command::new(exe);
    if let Some(css) = &global.css {
        command.arg("--css").arg(css);
    }
    if let Some(theme) = &global.theme {
        command.arg("--theme").arg(theme);
    }
    for (enabled, flag) in [
        (global.allow_html, "--allow-html"),
        (global.allow_remote, "--allow-remote"),
        (global.timing, "--timing"),
    ] {
        if enabled {
            command.arg(flag);
        }
    }
    command
        .arg("--")
        .arg(file)
        .spawn()
        .with_context(|| format!("cannot open {}", file.display()))?;
    Ok(())
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
