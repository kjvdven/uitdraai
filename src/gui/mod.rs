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

use crate::watch;

const APP_ID: &str = "io.github.kjvdven.uitdraai";

type PathHook = Box<dyn Fn(&Path) -> Result<()> + Send + Sync>;

/// Work the window hands off to the rest of the app. Rendering and export run off the GTK thread.
pub struct Hooks {
    /// Renders the complete page, for the first load and `Ctrl+R`.
    pub render_page: Box<dyn Fn() -> Result<String> + Send + Sync>,
    /// Renders only the `#content` HTML, for live reload after a save.
    pub render_content: Box<dyn Fn() -> Result<String> + Send + Sync>,
    /// Exports the file to PDF at the given path.
    pub export_pdf: PathHook,
    /// Opens another Markdown file in a new window.
    pub open: PathHook,
}

/// Opens the preview window for `file` and blocks until it is closed.
///
/// Shows `page` first, then swaps in fresh content after every save.
pub fn run(page: String, file: PathBuf, hooks: Hooks) -> Result<()> {
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
    let default_pdf = file.with_extension("pdf");

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
        preview.load(&page);

        let status = StatusBar::new();
        status.set(&format!("Opened {}", clock()));
        preview.webview.set_vexpand(true);
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.append(&preview.webview);
        layout.append(&status.root);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(title.as_str())
            .default_width(900)
            .default_height(1000)
            .child(&layout)
            .build();
        add_shortcuts(app, &window, &preview, &status, &hooks, &default_pdf);
        window.present();

        let rx = rx.clone();
        glib::spawn_future_local(async move {
            while let Ok((content, took)) = rx.recv().await {
                match content {
                    Ok(content) => {
                        preview.replace_content(&content);
                        status.set(&format!("Updated {} · {}", clock(), millis(took)));
                    }
                    Err(err) => status.set(&format!("Error: {err:#}")),
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

fn add_shortcuts(
    app: &gtk::Application,
    window: &gtk::ApplicationWindow,
    preview: &Preview,
    status: &StatusBar,
    hooks: &Arc<Hooks>,
    default_pdf: &Path,
) {
    let reload = {
        let (preview, status, hooks) = (preview.clone(), status.clone(), Arc::clone(hooks));
        gio::ActionEntry::builder("reload")
            .activate(move |_: &gtk::ApplicationWindow, _, _| {
                let (preview, status, hooks) =
                    (preview.clone(), status.clone(), Arc::clone(&hooks));
                glib::spawn_future_local(async move {
                    match gio::spawn_blocking(move || (hooks.render_page)()).await {
                        Ok(Ok(page)) => {
                            preview.load(&page);
                            status.set(&format!("Reloaded {}", clock()));
                        }
                        Ok(Err(err)) => status.set(&format!("Error: {err:#}")),
                        Err(_) => status.set("Error: rendering panicked"),
                    }
                });
            })
            .build()
    };
    let export_pdf = {
        let (status, hooks, default_pdf) =
            (status.clone(), Arc::clone(hooks), default_pdf.to_owned());
        gio::ActionEntry::builder("export-pdf")
            .activate(move |window: &gtk::ApplicationWindow, _, _| {
                let mut dialog = gtk::FileDialog::builder().title("Export PDF");
                if let Some(dir) = default_pdf.parent() {
                    dialog = dialog.initial_folder(&gio::File::for_path(dir));
                }
                if let Some(name) = default_pdf.file_name() {
                    dialog = dialog.initial_name(name.to_string_lossy().as_ref());
                }
                let (status, hooks) = (status.clone(), Arc::clone(&hooks));
                dialog
                    .build()
                    .save(Some(window), None::<&gio::Cancellable>, move |result| {
                        // Err also means the dialog was cancelled; nothing to report then.
                        let Some(pdf) = result.ok().and_then(|file| file.path()) else {
                            return;
                        };
                        status.set("Exporting PDF…");
                        glib::spawn_future_local(async move {
                            let target = pdf.clone();
                            match gio::spawn_blocking(move || (hooks.export_pdf)(&target)).await {
                                Ok(Ok(())) => {
                                    status.set(&format!(
                                        "PDF saved {}: {}",
                                        clock(),
                                        pdf.display()
                                    ));
                                    status.show_pdf(pdf);
                                }
                                Ok(Err(err)) => status.set(&format!("Error: {err:#}")),
                                Err(_) => status.set("Error: export panicked"),
                            }
                        });
                    });
            })
            .build()
    };
    let open = {
        let (status, hooks) = (status.clone(), Arc::clone(hooks));
        gio::ActionEntry::builder("open")
            .activate(move |window: &gtk::ApplicationWindow, _, _| {
                let filter = gtk::FileFilter::new();
                filter.set_name(Some("Markdown"));
                filter.add_suffix("md");
                filter.add_suffix("markdown");
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                let dialog = gtk::FileDialog::builder()
                    .title("Open Markdown file")
                    .filters(&filters)
                    .build();
                let (status, hooks) = (status.clone(), Arc::clone(&hooks));
                dialog.open(Some(window), None::<&gio::Cancellable>, move |result| {
                    // Err also means the dialog was cancelled; nothing to report then.
                    let Some(path) = result.ok().and_then(|file| file.path()) else {
                        return;
                    };
                    if let Err(err) = (hooks.open)(&path) {
                        status.set(&format!("Error: {err:#}"));
                    }
                });
            })
            .build()
    };
    let toggle_bars = {
        let status = status.clone();
        gio::ActionEntry::builder("toggle-bars")
            .activate(move |_: &gtk::ApplicationWindow, _, _| {
                status.root.set_visible(!status.root.is_visible());
            })
            .build()
    };
    let zoom = |name: &str, factor: Option<f64>| {
        let webview = preview.webview.clone();
        gio::ActionEntry::builder(name)
            .activate(move |_: &gtk::ApplicationWindow, _, _| {
                let level = factor.map_or(1.0, |f| (webview.zoom_level() * f).clamp(0.3, 5.0));
                webview.set_zoom_level(level);
            })
            .build()
    };
    let close = gio::ActionEntry::builder("close")
        .activate(|window: &gtk::ApplicationWindow, _, _| window.close())
        .build();

    window.add_action_entries([
        reload,
        export_pdf,
        open,
        toggle_bars,
        zoom("zoom-in", Some(1.1)),
        zoom("zoom-out", Some(1.0 / 1.1)),
        zoom("zoom-reset", None),
        close,
    ]);
    app.set_accels_for_action("win.reload", &["<Control>r"]);
    app.set_accels_for_action("win.export-pdf", &["<Control>e"]);
    app.set_accels_for_action("win.open", &["<Control>o"]);
    app.set_accels_for_action("win.toggle-bars", &["<Control>t"]);
    // `+` needs Shift on most layouts, so Ctrl+= counts as zoom in too.
    app.set_accels_for_action(
        "win.zoom-in",
        &["<Control>plus", "<Control>equal", "<Control>KP_Add"],
    );
    app.set_accels_for_action("win.zoom-out", &["<Control>minus", "<Control>KP_Subtract"]);
    app.set_accels_for_action("win.zoom-reset", &["<Control>0", "<Control>KP_0"]);
    app.set_accels_for_action("win.close", &["<Control>q"]);
}

/// Bottom bar with the last event, plus buttons for the last exported PDF.
#[derive(Clone)]
struct StatusBar {
    root: gtk::Box,
    label: gtk::Label,
    pdf_buttons: gtk::Box,
    last_pdf: Rc<RefCell<Option<PathBuf>>>,
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
        let open = gtk::Button::builder()
            .label("Open PDF")
            .css_classes(["flat"])
            .build();
        let reveal = gtk::Button::builder()
            .label("Show in folder")
            .css_classes(["flat"])
            .build();
        let pdf_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        pdf_buttons.append(&open);
        pdf_buttons.append(&reveal);
        // Invisible rather than hidden until the first export, so the bar keeps its height.
        pdf_buttons.set_opacity(0.0);
        pdf_buttons.set_sensitive(false);

        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        root.append(&label);
        root.append(&pdf_buttons);

        let bar = Self {
            root,
            label,
            pdf_buttons,
            last_pdf: Rc::new(RefCell::new(None)),
        };
        let this = bar.clone();
        open.connect_clicked(move |_| this.launch_pdf(false));
        let this = bar.clone();
        reveal.connect_clicked(move |_| this.launch_pdf(true));
        bar
    }

    fn set(&self, text: &str) {
        self.label.set_label(text);
        self.label.set_tooltip_text(Some(text));
    }

    fn show_pdf(&self, pdf: PathBuf) {
        self.last_pdf.replace(Some(pdf));
        self.pdf_buttons.set_opacity(1.0);
        self.pdf_buttons.set_sensitive(true);
    }

    /// Opens the last PDF in its default app, or its folder in the file manager.
    fn launch_pdf(&self, in_folder: bool) {
        let Some(pdf) = self.last_pdf.borrow().clone() else {
            return;
        };
        let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(&pdf)));
        let this = self.clone();
        let done = move |result: Result<(), glib::Error>| {
            if let Err(err) = result {
                this.set(&format!("Error: cannot open {}: {err}", pdf.display()));
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
        let script = format!("document.getElementById('content').innerHTML = {literal};");
        self.webview.evaluate_javascript(
            &script,
            None,
            None,
            None::<&gio::Cancellable>,
            |result| {
                if let Err(err) = result {
                    eprintln!("Error: cannot update preview: {err}");
                }
            },
        );
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
