mod render;
mod theme;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
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
        Some(Command::Themes { dump: Some(name) }) => {
            print!("{}", theme::load(&name, theme_dir.as_deref())?);
        }
        Some(Command::Themes { dump: None }) => {
            for name in theme::list(theme_dir.as_deref())? {
                println!("{name}");
            }
        }
        None => bail!("the preview window is not implemented yet; use `uitdraai render`"),
    }
    Ok(())
}

fn render_file(file: &Path, global: &GlobalOpts, theme_dir: Option<&Path>) -> Result<String> {
    let markdown =
        fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
    let css = theme::resolve(global.css.as_deref(), global.theme.as_deref(), theme_dir)?;
    let opts = RenderOptions {
        allow_html: global.allow_html,
        allow_remote: global.allow_remote,
    };
    Ok(render::render_page(&markdown, &css, opts))
}
