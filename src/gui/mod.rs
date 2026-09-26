use std::cell::Cell;
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Result, bail};
use webkit6::prelude::*;
use webkit6::{
    NavigationPolicyDecision, PolicyDecision, PolicyDecisionType, Settings, WebView, gio, glib, gtk,
};

use crate::watch;

const APP_ID: &str = "io.github.kjvdven.uitdraai";

/// Opens the preview window for `file` and blocks until it is closed.
///
/// Shows `page` first, then swaps in the output of `render_content` after every save.
/// Rendering runs on a watcher thread; only the finished HTML reaches the GTK thread.
pub fn run(
    page: String,
    file: PathBuf,
    render_content: impl Fn() -> Result<String> + Send + 'static,
) -> Result<()> {
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

    let (tx, rx) = async_channel::unbounded();
    thread::spawn(move || {
        let result = watch::watch(&file, || {
            // Only fails once the window is gone, and then nobody needs the update.
            let _ = tx.send_blocking(render_content());
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
        let webview = new_webview(&base_uri);
        webview.load_html(&page, Some(&base_uri));
        gtk::ApplicationWindow::builder()
            .application(app)
            .title(title.as_str())
            .default_width(900)
            .default_height(1000)
            .child(&webview)
            .build()
            .present();

        let rx = rx.clone();
        glib::spawn_future_local(async move {
            while let Ok(result) = rx.recv().await {
                match result {
                    Ok(content) => replace_content(&webview, &content),
                    Err(err) => eprintln!("Error: {err:#}"),
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

/// Swaps the inner HTML of `#content`, which keeps the scroll position.
fn replace_content(webview: &WebView, content: &str) {
    // serde_json turns the HTML into a safe JS string literal; never concatenate it raw.
    let Ok(literal) = serde_json::to_string(content) else {
        return;
    };
    let script = format!("document.getElementById('content').innerHTML = {literal};");
    webview.evaluate_javascript(&script, None, None, None::<&gio::Cancellable>, |result| {
        if let Err(err) = result {
            eprintln!("Error: cannot update preview: {err}");
        }
    });
}

/// Builds a locked-down WebView that only shows the page `load_html` gives it.
fn new_webview(base_uri: &str) -> WebView {
    let settings = Settings::builder()
        .enable_javascript_markup(false)
        .allow_file_access_from_file_urls(false)
        .allow_universal_access_from_file_urls(false)
        .build();
    let webview = WebView::builder().settings(&settings).build();

    // Its Reload and Back items would navigate away from our page.
    webview.connect_context_menu(|_, _, _| true);

    // The load_html call right after this is the only navigation we start ourselves.
    // Matching on URI can't tell it apart: a reload requests the very same base URI.
    let expect_load = Cell::new(true);
    let base_uri = base_uri.to_owned();
    webview.connect_decide_policy(move |_, decision, decision_type| {
        match decision_type {
            PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction => {
                let uri = navigation_uri(decision).unwrap_or_default();
                if decision_type == PolicyDecisionType::NavigationAction
                    && expect_load.replace(false)
                {
                    decision.use_();
                } else if uri.starts_with(&format!("{base_uri}#")) {
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
    webview
}

fn navigation_uri(decision: &PolicyDecision) -> Option<String> {
    let decision = decision.downcast_ref::<NavigationPolicyDecision>()?;
    let uri = decision.navigation_action()?.request()?.uri()?;
    Some(uri.to_string())
}
