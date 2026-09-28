use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use webkit6::prelude::*;
use webkit6::{
    NavigationPolicyDecision, PolicyDecision, PolicyDecisionType, Settings, WebView, gio, glib, gtk,
};

use crate::config::{self, ExportFormat};
use crate::render::Rendered;
use crate::watch;

const APP_ID: &str = "io.github.kjvdven.uitdraai";
const CUSTOM_CSS_LABEL: &str = "Custom CSS";

type PageHook = Box<dyn Fn(Option<&str>) -> Result<Rendered> + Send + Sync>;
type CssHook = Box<dyn Fn(Option<&str>) -> Result<String> + Send + Sync>;
type ExportHook = Box<dyn Fn(&Path, Option<&str>) -> Result<()> + Send + Sync>;
type PathHook = Box<dyn Fn(&Path) -> Result<()> + Send + Sync>;

/// Work the window hands off to the rest of the app. Rendering and export run off the GTK thread.
///
/// A theme of `None` means the stylesheet from the command line (`--css` or `--theme`).
pub struct Hooks {
    /// Renders the complete page, for `Ctrl+R`.
    pub render_page: PageHook,
    /// Returns the stylesheet for a theme, for switching themes in place.
    pub theme_css: CssHook,
    /// Renders only the `#content` HTML, for live reload after a save.
    pub render_content: Box<dyn Fn() -> Result<Rendered> + Send + Sync>,
    /// Exports the file to PDF at the given path.
    pub export_pdf: ExportHook,
    /// Writes the complete HTML page to the given path.
    pub export_html: ExportHook,
    /// Opens another Markdown file in a new window.
    pub open: PathHook,
}

/// What the window starts with.
pub struct Options {
    /// The page rendered with the command-line stylesheet.
    pub page: Rendered,
    /// Theme names for the dropdown.
    pub themes: Vec<String>,
    /// The theme to preselect; `None` when `--css` is in use.
    pub theme: Option<String>,
    /// Where the export dialog starts: folder and file name.
    pub default_pdf: PathBuf,
    /// Command for "Open in editor"; `None` uses the desktop's default app.
    pub editor: Option<Vec<String>>,
    /// What `Ctrl+E` and the main export button produce.
    pub default_export: ExportFormat,
    /// The config file `Ctrl+,` opens.
    pub config_path: Option<PathBuf>,
    /// Show the toolbar and status bar (`--no-toolbar` turns them off).
    pub show_bars: bool,
}

/// Sets up the process for WebKit. Call first, before GTK starts or any thread exists.
///
/// `disable_dmabuf` works around an empty window on some GPUs (NVIDIA).
pub fn prepare(disable_dmabuf: bool) {
    if disable_dmabuf {
        // SAFETY: the caller runs this before GTK starts or any thread is spawned,
        // so nothing can read the environment concurrently.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
}

/// Asks for a Markdown file before any window exists; `None` when cancelled.
pub fn pick_file() -> Result<Option<PathBuf>> {
    let picked = Rc::new(RefCell::new(None));
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let result = Rc::clone(&picked);
    app.connect_activate(move |app| {
        // There is no window yet, so hold the app open until the dialog answers.
        let hold = app.hold();
        let result = Rc::clone(&result);
        markdown_dialog().open(
            None::<&gtk::Window>,
            None::<&gio::Cancellable>,
            move |file| {
                result.replace(file.ok().and_then(|file| file.path()));
                drop(hold);
            },
        );
    });
    let status = app.run_with_args::<&str>(&[]);
    if status != glib::ExitCode::SUCCESS {
        bail!("file dialog exited with {status:?}");
    }
    Ok(picked.take())
}

/// A file dialog that only shows Markdown files.
fn markdown_dialog() -> gtk::FileDialog {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Markdown"));
    filter.add_suffix("md");
    filter.add_suffix("markdown");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    gtk::FileDialog::builder()
        .title("Open Markdown file")
        .filters(&filters)
        .build()
}

/// Opens the preview window for `file` and blocks until it is closed.
///
/// Shows the initial page first, then swaps in fresh content after every save.
pub fn run(file: PathBuf, options: Options, hooks: Hooks) -> Result<()> {
    let base_dir = file
        .parent()
        .context("Markdown file has no parent directory")?;
    // Trailing slash, or the last path segment is treated as a file and dropped.
    let base_uri = format!(
        "{}/",
        glib::filename_to_uri(base_dir, None).context("cannot build base URI")?
    );
    let title = file.file_name().map_or_else(
        || "uitdraai".into(),
        |name| name.to_string_lossy().into_owned(),
    );
    let hooks = Arc::new(hooks);
    let markdown_file = file.clone();

    let (tx, rx) = async_channel::unbounded();
    let worker_hooks = Arc::clone(&hooks);
    thread::spawn(move || {
        let result = watch::watch(&file, || {
            let start = Instant::now();
            let content = (worker_hooks.render_content)();
            // Only fails once the window is gone, and then nobody needs the update.
            let _ = tx.send_blocking((content, start.elapsed()));
        });
        if let Err(err) = result {
            eprintln!("Error: {err:#}");
        }
    });

    // NON_UNIQUE: a second `uitdraai other.md` would otherwise just activate this process.
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        let preview = Preview::new(&base_uri);
        preview.load(&options.page.html);
        let status = StatusBar::new();
        status.set(&format!("Opened {}", clock()));

        // `None` stands for the --css stylesheet, listed first when it is in use.
        let mut choices: Vec<Option<String>> = options.themes.iter().cloned().map(Some).collect();
        if options.theme.is_none() {
            choices.insert(0, None);
        }
        let selected = choices
            .iter()
            .position(|choice| *choice == options.theme)
            .unwrap_or(0);
        let labels: Vec<&str> = choices
            .iter()
            .map(|choice| choice.as_deref().unwrap_or(CUSTOM_CSS_LABEL))
            .collect();
        let theme_picker = gtk::DropDown::from_strings(&labels);
        theme_picker.set_selected(u32::try_from(selected).unwrap_or(0));
        theme_picker.set_tooltip_text(Some("Theme"));
        let editor_button = gtk::Button::builder()
            .label("Open in editor")
            .action_name("win.open-editor")
            .hexpand(true)
            .halign(gtk::Align::End)
            .build();
        // A split button: the main part exports the default format, the arrow offers all.
        let export_button = gtk::Button::builder()
            .label(format!("Export {}", options.default_export.label()))
            .action_name("win.export")
            .build();
        let export_formats = gio::Menu::new();
        export_formats.append(Some("Export _PDF"), Some("win.export-pdf"));
        export_formats.append(Some("Export _HTML"), Some("win.export-html"));
        let export_menu = gtk::MenuButton::builder()
            .menu_model(&export_formats)
            .tooltip_text("Choose export format")
            .build();
        let export = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["linked"])
            .build();
        export.append(&export_button);
        export.append(&export_menu);
        let toolbar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        toolbar.append(&theme_picker);
        toolbar.append(&editor_button);
        toolbar.append(&export);

        toolbar.set_visible(options.show_bars);
        status.root.set_visible(options.show_bars);
        preview.webview.set_vexpand(true);
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.append(&toolbar);
        layout.append(&preview.webview);
        layout.append(&status.root);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(title.as_str())
            .default_width(900)
            .default_height(1000)
            .child(&layout)
            .build();
        let ctx = Ctx {
            preview,
            status,
            toolbar,
            export_menu,
            hooks: Arc::clone(&hooks),
            markdown_file: markdown_file.clone(),
            default_pdf: options.default_pdf.clone(),
            editor: options.editor.clone(),
            default_export: options.default_export,
            config_path: options.config_path.clone(),
            theme: Rc::new(RefCell::new(options.theme.clone())),
        };
        {
            let ctx = ctx.clone();
            theme_picker.connect_selected_notify(move |picker| {
                let index = usize::try_from(picker.selected()).unwrap_or(0);
                if let Some(choice) = choices.get(index) {
                    ctx.theme.replace(choice.clone());
                    ctx.apply_theme();
                }
            });
        }
        add_shortcuts(app, &window, &ctx);
        window.present();

        let rx = rx.clone();
        glib::spawn_future_local(async move {
            while let Ok((content, took)) = rx.recv().await {
                match content {
                    Ok(content) => {
                        ctx.preview.replace_content(&content.html);
                        ctx.status
                            .set(&format!("Updated {} · {}", clock(), millis(took)));
                    }
                    Err(err) => ctx.status.set(&format!("Error: {err:#}")),
                }
            }
        });
    });

    // Our own args are already parsed by clap; GTK must not see them.
    let status = app.run_with_args::<&str>(&[]);
    if status != glib::ExitCode::SUCCESS {
        bail!("preview window exited with {status:?}");
    }
    Ok(())
}

/// Everything the window's actions work on.
#[derive(Clone)]
struct Ctx {
    preview: Preview,
    status: StatusBar,
    toolbar: gtk::Box,
    export_menu: gtk::MenuButton,
    hooks: Arc<Hooks>,
    markdown_file: PathBuf,
    default_pdf: PathBuf,
    editor: Option<Vec<String>>,
    default_export: ExportFormat,
    config_path: Option<PathBuf>,
    theme: Rc<RefCell<Option<String>>>,
}

impl Ctx {
    /// Renders the whole page again with the current theme; loses the scroll position.
    fn reload(&self) {
        let ctx = self.clone();
        let theme = self.theme.borrow().clone();
        glib::spawn_future_local(async move {
            let hooks = Arc::clone(&ctx.hooks);
            match gio::spawn_blocking(move || (hooks.render_page)(theme.as_deref())).await {
                Ok(Ok(page)) => {
                    ctx.preview.load(&page.html);
                    ctx.status.set(&format!("Reloaded {}", clock()));
                }
                Ok(Err(err)) => ctx.status.set(&format!("Error: {err:#}")),
                Err(_) => ctx.status.set("Error: rendering panicked"),
            }
        });
    }

    /// Restyles the page with the current theme without reloading it.
    fn apply_theme(&self) {
        let ctx = self.clone();
        let theme = self.theme.borrow().clone();
        glib::spawn_future_local(async move {
            let hooks = Arc::clone(&ctx.hooks);
            let name = theme.clone().unwrap_or_else(|| CUSTOM_CSS_LABEL.to_owned());
            match gio::spawn_blocking(move || (hooks.theme_css)(theme.as_deref())).await {
                Ok(Ok(css)) => {
                    ctx.preview.replace_css(&css);
                    ctx.status.set(&format!("Theme {name} {}", clock()));
                }
                Ok(Err(err)) => ctx.status.set(&format!("Error: {err:#}")),
                Err(_) => ctx.status.set("Error: loading theme panicked"),
            }
        });
    }

    /// Asks where to save, then exports to `format` with the current theme.
    fn export(&self, window: &gtk::ApplicationWindow, format: ExportFormat) {
        let label = format.label();
        let default = self.default_pdf.with_extension(format.extension());
        let mut dialog = gtk::FileDialog::builder().title(format!("Export {label}"));
        if let Some(dir) = default.parent() {
            dialog = dialog.initial_folder(&gio::File::for_path(dir));
        }
        if let Some(name) = default.file_name() {
            dialog = dialog.initial_name(name.to_string_lossy().as_ref());
        }
        let ctx = self.clone();
        dialog
            .build()
            .save(Some(window), None::<&gio::Cancellable>, move |result| {
                // Err also means the dialog was cancelled; nothing to report then.
                let Some(path) = result.ok().and_then(|file| file.path()) else {
                    return;
                };
                ctx.status.set(&format!("Exporting {label}…"));
                let theme = ctx.theme.borrow().clone();
                glib::spawn_future_local(async move {
                    let (hooks, target) = (Arc::clone(&ctx.hooks), path.clone());
                    let export = move || {
                        let hook = match format {
                            ExportFormat::Pdf => &hooks.export_pdf,
                            ExportFormat::Html => &hooks.export_html,
                        };
                        hook(&target, theme.as_deref())
                    };
                    match gio::spawn_blocking(export).await {
                        Ok(Ok(())) => {
                            ctx.status.set(&format!(
                                "{label} saved {}: {}",
                                clock(),
                                path.display()
                            ));
                            ctx.status.show_export(path, format);
                        }
                        Ok(Err(err)) => ctx.status.set(&format!("Error: {err:#}")),
                        Err(_) => ctx.status.set("Error: export panicked"),
                    }
                });
            });
    }

    /// Asks for a Markdown file and opens it in a new window.
    fn open(&self, window: &gtk::ApplicationWindow) {
        let ctx = self.clone();
        markdown_dialog().open(Some(window), None::<&gio::Cancellable>, move |result| {
            // Err also means the dialog was cancelled; nothing to report then.
            let Some(path) = result.ok().and_then(|file| file.path()) else {
                return;
            };
            if let Err(err) = (ctx.hooks.open)(&path) {
                ctx.status.set(&format!("Error: {err:#}"));
            }
        });
    }

    fn open_editor(&self, window: &gtk::ApplicationWindow) {
        self.open_in_editor(window, &self.markdown_file);
    }

    /// Opens the config file in the editor, creating it from a template first.
    fn open_config(&self, window: &gtk::ApplicationWindow) {
        let Some(path) = self.config_path.clone() else {
            self.status.set("Error: no config directory found");
            return;
        };
        if let Err(err) = config::ensure_exists(&path) {
            self.status.set(&format!("Error: {err:#}"));
            return;
        }
        if self.open_in_editor(window, &path) {
            self.status.set("Config changes apply to new windows");
        }
    }

    /// Opens `path` with the configured editor command, or the desktop's default app.
    /// Returns whether the editor started; errors go to the status bar.
    fn open_in_editor(&self, window: &gtk::ApplicationWindow, path: &Path) -> bool {
        let result = match self.editor.as_deref() {
            // No `--` before the path: editors differ in support, and the path is
            // absolute, so it can't be read as an option.
            Some([program, args @ ..]) => std::process::Command::new(program)
                .args(args)
                .arg(path)
                .spawn()
                .map(drop)
                .with_context(|| format!("cannot start {program}")),
            // Not gtk::FileLauncher: its portal shows a chooser of "recommended" apps that
            // can leave out the default one the user picked.
            _ => {
                let uri = gio::File::for_path(path).uri();
                let context = WidgetExt::display(window).app_launch_context();
                gio::AppInfo::launch_default_for_uri(&uri, Some(&context)).context("no default app")
            }
        };
        match result {
            Ok(()) => true,
            Err(err) => {
                self.status
                    .set(&format!("Error: cannot open editor: {err:#}"));
                false
            }
        }
    }

    fn toggle_bars(&self) {
        let visible = !self.toolbar.is_visible();
        self.toolbar.set_visible(visible);
        self.status.root.set_visible(visible);
    }

    fn zoom(&self, factor: Option<f64>) {
        let webview = &self.preview.webview;
        let level = factor.map_or(1.0, |f| (webview.zoom_level() * f).clamp(0.3, 5.0));
        webview.set_zoom_level(level);
    }
}

/// Window actions with their keys and help text; the one list behind both the
/// accelerators and the `Ctrl+?` help window. The first key is the one shown in help.
const SHORTCUTS: &[(&str, &[&str], &str)] = &[
    ("win.open", &["<Control>o"], "Open another file"),
    ("win.open-editor", &["<Control><Shift>o"], "Open in editor"),
    ("win.open-config", &["<Control>comma"], "Open config file"),
    ("win.export", &["<Control>e"], "Export (default format)"),
    (
        "win.export-menu",
        &["<Control><Shift>e"],
        "Choose export format",
    ),
    ("win.reload", &["<Control>r"], "Reload page"),
    (
        "win.toggle-bars",
        &["<Control>t"],
        "Show or hide toolbar and status bar",
    ),
    // `+` needs Shift on most layouts, so Ctrl+= counts as zoom in too.
    (
        "win.zoom-in",
        &["<Control>plus", "<Control>equal", "<Control>KP_Add"],
        "Zoom in",
    ),
    (
        "win.zoom-out",
        &["<Control>minus", "<Control>KP_Subtract"],
        "Zoom out",
    ),
    (
        "win.zoom-reset",
        &["<Control>0", "<Control>KP_0"],
        "Reset zoom",
    ),
    ("win.shortcuts", &["<Control>question"], "Show shortcuts"),
    ("win.close", &["<Control>q"], "Close window"),
];

fn add_shortcuts(app: &gtk::Application, window: &gtk::ApplicationWindow, ctx: &Ctx) {
    let action = |name: &str, run: fn(&Ctx, &gtk::ApplicationWindow)| {
        let ctx = ctx.clone();
        gio::ActionEntry::builder(name)
            .activate(move |window: &gtk::ApplicationWindow, _, _| run(&ctx, window))
            .build()
    };
    window.add_action_entries([
        action("reload", |ctx, _| ctx.reload()),
        action("export", |ctx, window| {
            ctx.export(window, ctx.default_export)
        }),
        action("export-pdf", |ctx, window| {
            ctx.export(window, ExportFormat::Pdf)
        }),
        action("export-html", |ctx, window| {
            ctx.export(window, ExportFormat::Html)
        }),
        action("export-menu", |ctx, _| ctx.export_menu.popup()),
        action("open", Ctx::open),
        action("open-editor", Ctx::open_editor),
        action("open-config", Ctx::open_config),
        action("toggle-bars", |ctx, _| ctx.toggle_bars()),
        action("zoom-in", |ctx, _| ctx.zoom(Some(1.1))),
        action("zoom-out", |ctx, _| ctx.zoom(Some(1.0 / 1.1))),
        action("zoom-reset", |ctx, _| ctx.zoom(None)),
        action("shortcuts", |_, window| show_shortcuts(window)),
        action("close", |_, window| window.close()),
    ]);
    for (action, keys, _) in SHORTCUTS {
        app.set_accels_for_action(action, keys);
    }
}

/// A small modal list of all shortcuts; Esc closes it.
fn show_shortcuts(parent: &gtk::ApplicationWindow) {
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(24)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();
    for (row, (_, keys, help)) in (0..).zip(SHORTCUTS) {
        let key = keys
            .first()
            .and_then(|accel| gtk::accelerator_parse(*accel))
            .map(|(key, mods)| gtk::accelerator_get_label(key, mods).to_string())
            .unwrap_or_default();
        let key = gtk::Label::builder()
            .label(key)
            .xalign(1.0)
            .css_classes(["dim-label"])
            .build();
        grid.attach(&key, 0, row, 1, 1);
        grid.attach(
            &gtk::Label::builder().label(*help).xalign(0.0).build(),
            1,
            row,
            1,
            1,
        );
    }

    let escape = gtk::ShortcutController::new();
    escape.add_shortcut(gtk::Shortcut::new(
        gtk::ShortcutTrigger::parse_string("Escape"),
        Some(gtk::NamedAction::new("window.close")),
    ));
    let help = gtk::Window::builder()
        .title("Keyboard shortcuts")
        .transient_for(parent)
        .modal(true)
        .resizable(false)
        .child(&grid)
        .build();
    help.add_controller(escape);
    help.present();
}

/// Bottom bar with the last event, plus buttons for the last exported file.
#[derive(Clone)]
struct StatusBar {
    root: gtk::Box,
    label: gtk::Label,
    export_buttons: gtk::Box,
    open_button: gtk::Button,
    last_export: Rc<RefCell<Option<PathBuf>>>,
}

impl StatusBar {
    fn new() -> Self {
        // Middle ellipsis keeps the file name visible and stops long paths widening the window.
        let label = gtk::Label::builder()
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .css_classes(["dim-label"])
            .build();
        let open_button = gtk::Button::builder()
            .label("Open PDF")
            .css_classes(["flat"])
            .build();
        let reveal = gtk::Button::builder()
            .label("Show in folder")
            .css_classes(["flat"])
            .build();
        let export_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        export_buttons.append(&open_button);
        export_buttons.append(&reveal);
        // Invisible rather than hidden until the first export, so the bar keeps its height.
        export_buttons.set_opacity(0.0);
        export_buttons.set_sensitive(false);

        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        root.append(&label);
        root.append(&export_buttons);

        let bar = Self {
            root,
            label,
            export_buttons,
            open_button: open_button.clone(),
            last_export: Rc::new(RefCell::new(None)),
        };
        let this = bar.clone();
        open_button.connect_clicked(move |_| this.launch_export(false));
        let this = bar.clone();
        reveal.connect_clicked(move |_| this.launch_export(true));
        bar
    }

    fn set(&self, text: &str) {
        self.label.set_label(text);
        self.label.set_tooltip_text(Some(text));
    }

    fn show_export(&self, path: PathBuf, format: ExportFormat) {
        self.last_export.replace(Some(path));
        self.open_button
            .set_label(&format!("Open {}", format.label()));
        self.export_buttons.set_opacity(1.0);
        self.export_buttons.set_sensitive(true);
    }

    /// Opens the last export in its default app, or its folder in the file manager.
    fn launch_export(&self, in_folder: bool) {
        let Some(path) = self.last_export.borrow().clone() else {
            return;
        };
        let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(&path)));
        let this = self.clone();
        let done = move |result: Result<(), glib::Error>| {
            if let Err(err) = result {
                this.set(&format!("Error: cannot open {}: {err}", path.display()));
            }
        };
        let window = self.root.root().and_downcast::<gtk::Window>();
        if in_folder {
            launcher.open_containing_folder(window.as_ref(), None::<&gio::Cancellable>, done);
        } else {
            launcher.launch(window.as_ref(), None::<&gio::Cancellable>, done);
        }
    }
}

/// The WebView plus what it needs to only ever show our own page.
#[derive(Clone)]
struct Preview {
    webview: WebView,
    base_uri: String,
    /// Lets exactly one navigation through: the one `load` starts.
    expect_load: Rc<Cell<bool>>,
}

impl Preview {
    /// Builds a locked-down WebView that only shows pages passed to `load`.
    fn new(base_uri: &str) -> Self {
        let settings = Settings::builder()
            .enable_javascript_markup(false)
            .allow_file_access_from_file_urls(false)
            .allow_universal_access_from_file_urls(false)
            .build();
        let webview = WebView::builder().settings(&settings).build();

        // Its Reload and Back items would navigate away from our page.
        webview.connect_context_menu(|_, _, _| true);

        // Matching on URI can't tell our load apart: a reload requests the very same base URI.
        let expect_load = Rc::new(Cell::new(false));
        let (flag, base) = (Rc::clone(&expect_load), base_uri.to_owned());
        webview.connect_decide_policy(move |_, decision, decision_type| {
            match decision_type {
                PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction => {
                    let uri = navigation_uri(decision).unwrap_or_default();
                    if decision_type == PolicyDecisionType::NavigationAction && flag.replace(false)
                    {
                        decision.use_();
                    } else if uri.starts_with(&format!("{base}#")) {
                        // Footnotes and other in-page anchors.
                        decision.use_();
                    } else {
                        decision.ignore();
                        if uri.starts_with("https://") || uri.starts_with("http://") {
                            gtk::UriLauncher::new(&uri).launch(
                                None::<&gtk::Window>,
                                None::<&gio::Cancellable>,
                                |_| {},
                            );
                        }
                    }
                }
                _ => decision.use_(),
            }
            true
        });

        Self {
            webview,
            base_uri: base_uri.to_owned(),
            expect_load,
        }
    }

    /// Replaces the whole page; loses the scroll position.
    fn load(&self, page: &str) {
        self.expect_load.set(true);
        self.webview.load_html(page, Some(&self.base_uri));
    }

    /// Swaps the inner HTML of `#content`, which keeps the scroll position.
    fn replace_content(&self, content: &str) {
        // serde_json turns the HTML into a safe JS string literal; never concatenate it raw.
        let Ok(literal) = serde_json::to_string(content) else {
            return;
        };
        self.run_script(&format!(
            "document.getElementById('content').innerHTML = {literal};"
        ));
    }

    /// Swaps the page's stylesheet and keeps roughly the same place in the document.
    fn replace_css(&self, css: &str) {
        let Ok(literal) = serde_json::to_string(css) else {
            return;
        };
        // A new theme changes the page height, so keep the relative position, not pixels.
        // The block scope lets the script run again without redeclaring its consts.
        self.run_script(&format!(
            "{{ const room = () => Math.max(1, document.documentElement.scrollHeight - innerHeight);
               const at = scrollY / room();
               document.querySelector('head style').textContent = {literal};
               scrollTo(0, at * room()); }}"
        ));
    }

    fn run_script(&self, script: &str) {
        self.webview
            .evaluate_javascript(script, None, None, None::<&gio::Cancellable>, |result| {
                if let Err(err) = result {
                    eprintln!("Error: cannot update preview: {err}");
                }
            });
    }
}

fn navigation_uri(decision: &PolicyDecision) -> Option<String> {
    let decision = decision.downcast_ref::<NavigationPolicyDecision>()?;
    let uri = decision.navigation_action()?.request()?.uri()?;
    Some(uri.to_string())
}

fn clock() -> String {
    glib::DateTime::now_local()
        .and_then(|now| now.format("%H:%M:%S"))
        .map(|time| time.to_string())
        .unwrap_or_default()
}

fn millis(duration: Duration) -> String {
    format!("{} ms", duration.as_millis())
}
