use std::cell::Cell;
use std::path::PathBuf;
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

/// Rendering work the window hands off to the rest of the app. Both run off the GTK thread.
pub struct Hooks {
    /// Renders the complete page, for the first load and `Ctrl+R`.
    pub render_page: Box<dyn Fn() -> Result<String> + Send + Sync>,
    /// Renders only the `#content` HTML, for live reload after a save.
    pub render_content: Box<dyn Fn() -> Result<String> + Send + Sync>,
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

        let status = gtk::Label::builder()
            .label(format!("Opened {}", clock()))
            .xalign(0.0)
            .margin_start(12)
            .margin_end(12)
            .margin_top(6)
            .margin_bottom(6)
            .css_classes(["dim-label"])
            .build();
        preview.webview.set_vexpand(true);
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.append(&preview.webview);
        layout.append(&status);

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(title.as_str())
            .default_width(900)
            .default_height(1000)
            .child(&layout)
            .build();
        add_shortcuts(app, &window, &preview, &status, &hooks);
        window.present();

        let rx = rx.clone();
        glib::spawn_future_local(async move {
            while let Ok((content, took)) = rx.recv().await {
                match content {
                    Ok(content) => {
                        preview.replace_content(&content);
                        status.set_label(&format!("Updated {} · {}", clock(), millis(took)));
                    }
                    Err(err) => status.set_label(&format!("Error: {err:#}")),
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
    status: &gtk::Label,
    hooks: &Arc<Hooks>,
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
                            status.set_label(&format!("Reloaded {}", clock()));
                        }
                        Ok(Err(err)) => status.set_label(&format!("Error: {err:#}")),
                        Err(_) => status.set_label("Error: rendering panicked"),
                    }
                });
            })
            .build()
    };
    let toggle_bars = {
        let status = status.clone();
        gio::ActionEntry::builder("toggle-bars")
            .activate(move |_: &gtk::ApplicationWindow, _, _| {
                status.set_visible(!status.is_visible());
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
        toggle_bars,
        zoom("zoom-in", Some(1.1)),
        zoom("zoom-out", Some(1.0 / 1.1)),
        zoom("zoom-reset", None),
        close,
    ]);
    app.set_accels_for_action("win.reload", &["<Control>r"]);
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
