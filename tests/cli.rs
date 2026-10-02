use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uitdraai-cli-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs the binary with an empty config dir, so the user's own themes don't leak in.
fn uitdraai(config: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uitdraai"))
        .env("XDG_CONFIG_HOME", config)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn render_writes_page_to_stdout_without_scripts() {
    let dir = temp_dir("render");
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi\n\n<script>alert(1)</script>").unwrap();

    let out = uitdraai(&dir, &["render", md.to_str().unwrap()]);
    let html = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(html.contains("<h1 id=\"hi\">"));
    assert!(!html.contains("<script>"));
}

#[test]
fn render_turns_math_into_svg() {
    let dir = temp_dir("math");
    let out = uitdraai(&dir, &["render", "tests/fixtures/math.md"]);
    let html = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert_eq!(
        html.matches("<span class=\"math math-inline\"><svg ")
            .count(),
        2
    );
    assert_eq!(
        html.matches("<span class=\"math math-display\"><svg ")
            .count(),
        1
    );
    assert!(html.contains("<span data-math-style=\"inline\">\\frac{1}</span>"));
    assert!(html.contains("costs $5 and $10"));
}

#[test]
fn render_kitchen_sink() {
    let dir = temp_dir("kitchen-sink");
    let out = uitdraai(&dir, &["render", "tests/fixtures/kitchen-sink.md"]);
    let html = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(html.contains("<h6 id=\"heading-6\">"));
    assert!(html.contains("<span class=\"math math-display\"><svg "));
    assert!(html.contains("<img src=\"../../data/screenshot.png\""));
    assert!(!html.contains("<kbd>"));
}

#[test]
fn render_writes_to_output_file() {
    let dir = temp_dir("output");
    let md = dir.join("notes.md");
    let html = dir.join("notes.html");
    fs::write(&md, "# Hi").unwrap();

    let out = uitdraai(
        &dir,
        &["render", md.to_str().unwrap(), "-o", html.to_str().unwrap()],
    );
    assert!(out.status.success());
    assert!(
        fs::read_to_string(&html)
            .unwrap()
            .contains("<h1 id=\"hi\">")
    );
}

#[test]
fn themes_lists_user_themes_and_dumps_css() {
    let dir = temp_dir("themes");
    let themes = dir.join("uitdraai/themes");
    fs::create_dir_all(&themes).unwrap();
    fs::write(themes.join("zen.css"), "/* zen */").unwrap();

    let list = uitdraai(&dir, &["themes"]);
    assert_eq!(String::from_utf8(list.stdout).unwrap(), "default\nzen\n");

    let dump = uitdraai(&dir, &["themes", "--dump", "zen"]);
    assert_eq!(String::from_utf8(dump.stdout).unwrap(), "/* zen */");
}

#[test]
fn unknown_theme_fails() {
    let dir = temp_dir("unknown");
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi").unwrap();

    let out = uitdraai(&dir, &["render", md.to_str().unwrap(), "--theme", "nope"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("unknown theme 'nope'")
    );
}

#[test]
fn export_without_weasyprint_explains_how_to_install() {
    let dir = temp_dir("nopdf");
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_uitdraai"))
        .env("XDG_CONFIG_HOME", &dir)
        .env("PATH", "")
        .args(["export", md.to_str().unwrap(), "--pdf"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(!out.status.success());
    assert!(stderr.contains("weasyprint not found"));
    assert!(stderr.contains("sudo pacman -S python-weasyprint"));
}

#[test]
fn export_writes_pdf_next_to_markdown() {
    if Command::new("weasyprint")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipped: weasyprint not installed");
        return;
    }
    let dir = temp_dir("pdf");
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi\n\n![img](missing.png)").unwrap();

    let out = uitdraai(&dir, &["export", md.to_str().unwrap(), "--pdf"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        fs::read(dir.join("notes.pdf"))
            .unwrap()
            .starts_with(b"%PDF")
    );
}

#[test]
fn timing_logs_each_step_to_stderr() {
    let dir = temp_dir("timing");
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi").unwrap();

    let out = uitdraai(&dir, &["render", md.to_str().unwrap(), "--timing"]);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(out.status.success());
    for step in ["read", "theme", "render"] {
        assert!(stderr.contains(&format!("timing: {step} ")), "{stderr}");
    }
}

#[test]
fn config_theme_is_used_unless_a_flag_overrides_it() {
    let dir = temp_dir("config-theme");
    let app = dir.join("uitdraai");
    fs::create_dir_all(app.join("themes")).unwrap();
    fs::write(app.join("themes/zen.css"), "/* zen */").unwrap();
    fs::write(app.join("config.toml"), "theme = \"zen\"\n").unwrap();
    let md = dir.join("notes.md");
    fs::write(&md, "# Hi").unwrap();

    let out = uitdraai(&dir, &["render", md.to_str().unwrap()]);
    assert!(String::from_utf8(out.stdout).unwrap().contains("/* zen */"));

    let out = uitdraai(
        &dir,
        &["render", md.to_str().unwrap(), "--theme", "default"],
    );
    assert!(!String::from_utf8(out.stdout).unwrap().contains("/* zen */"));
}

#[test]
fn invalid_config_names_the_bad_key() {
    let dir = temp_dir("config-typo");
    fs::create_dir_all(dir.join("uitdraai")).unwrap();
    fs::write(dir.join("uitdraai/config.toml"), "thme = \"zen\"\n").unwrap();

    let out = uitdraai(&dir, &["themes"]);
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("thme"));
}
