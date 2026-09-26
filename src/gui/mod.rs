use std::cell::Cell;
use std::path::Path;

use anyhow::{Context, Result, bail};
use webkit6::prelude::*;
use webkit6::{
    NavigationPolicyDecision, PolicyDecision, PolicyDecisionType, Settings, WebView, gio, glib, gtk,
};

const APP_ID: &str = "io.github.kjvdven.uitdraai";

/// Opens the preview window showing `html` and blocks until it is closed.
///
/// Relative links and images resolve against `base_dir`, the Markdown file's directory.
pub fn run(html: String, base_dir: &Path, title: &str) -> Result<()> {
    // Trailing slash, or the last path segment is treated as a file and dropped.
    let base_uri = format!(
        "{}/",
        glib::filename_to_uri(base_dir, None).context("cannot build base URI")?
    );
    let title = title.to_owned();

    // NON_UNIQUE: a second `uitdraai other.md` would otherwise just activate this process.
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        let webview = new_webview(&base_uri);
        webview.load_html(&html, Some(&base_uri));
        gtk::ApplicationWindow::builder()
            .application(app)
            .title(title.as_str())
            .default_width(900)
            .default_height(1000)
            .child(&webview)
            .build()
            .present();
    });

    // Our own args are already parsed by clap; GTK must not see them.
    let status = app.run_with_args::<&str>(&[]);
    if status != glib::ExitCode::SUCCESS {
        bail!("preview window exited with {status:?}");
    }
    Ok(())
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
