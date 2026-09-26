use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;

const DEBOUNCE: Duration = Duration::from_millis(100);

/// Calls `on_change` after each save of `file`. Blocks until the watcher fails.
///
/// Watches the parent directory, because editors that save by renaming a temp file
/// over the original would silently end a watch on the file itself.
pub fn watch(file: &Path, mut on_change: impl FnMut()) -> Result<()> {
    let file = file
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", file.display()))?;
    let dir = file.parent().context("file has no parent directory")?;
    let name = file.file_name().context("path has no file name")?;

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(DEBOUNCE, tx).context("cannot start file watcher")?;
    debouncer
        .watcher()
        .watch(dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("cannot watch {}", dir.display()))?;

    // notify also reports opens, so our own reads would trigger endless re-renders.
    // Only a new modification time counts as a save.
    let mut last_modified = modified(&file);
    for result in rx {
        let events = result.context("file watcher failed")?;
        if !events
            .iter()
            .any(|event| event.path.file_name() == Some(name))
        {
            continue;
        }
        let now = modified(&file);
        if now.is_some() && now != last_modified {
            last_modified = now;
            on_change();
        }
    }
    Ok(())
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uitdraai-watch-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn start_watching(file: &Path) -> mpsc::Receiver<()> {
        let (tx, rx) = mpsc::channel();
        let file = file.to_owned();
        thread::spawn(move || watch(&file, || tx.send(()).unwrap()));
        // Give the watcher time to register before the test writes.
        thread::sleep(Duration::from_millis(200));
        rx
    }

    #[test]
    fn notices_atomic_save_via_rename_twice() {
        let dir = temp_dir("rename");
        let file = dir.join("notes.md");
        fs::write(&file, "one").unwrap();
        let changes = start_watching(&file);

        for content in ["two", "three"] {
            let tmp = dir.join(".notes.md.swp");
            fs::write(&tmp, content).unwrap();
            fs::rename(&tmp, &file).unwrap();
            changes.recv_timeout(Duration::from_secs(2)).unwrap();
        }
    }

    #[test]
    fn ignores_other_files_in_the_directory() {
        let dir = temp_dir("other");
        let file = dir.join("notes.md");
        fs::write(&file, "one").unwrap();
        let changes = start_watching(&file);

        fs::write(dir.join("other.md"), "x").unwrap();
        assert!(changes.recv_timeout(Duration::from_millis(500)).is_err());
    }

    #[test]
    fn ignores_reads_of_the_file() {
        let dir = temp_dir("read");
        let file = dir.join("notes.md");
        fs::write(&file, "one").unwrap();
        let changes = start_watching(&file);

        fs::read_to_string(&file).unwrap();
        assert!(changes.recv_timeout(Duration::from_millis(500)).is_err());
    }
}
