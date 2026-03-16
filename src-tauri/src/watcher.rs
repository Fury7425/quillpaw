use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

static WATCHER: OnceLock<Mutex<Option<RecommendedWatcher>>> = OnceLock::new();

/// Start watching a vault folder and emit change events to the frontend.
pub async fn start_watcher(app: AppHandle, vault_path: String) -> Result<(), String> {
    stop_watcher().await?;
    let app_handle = app.clone();
    let (tx, rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .map_err(|e| e.to_string())?;
    watcher
        .watch(Path::new(&vault_path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        for res in rx {
            if let Ok(event) = res {
                if event.paths.iter().any(|path| {
                    let path_string = path.to_string_lossy();
                    path_string.contains(".quillpaw") || path_string.contains(".assets")
                }) {
                    continue;
                }
                let payload = event
                    .paths
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>();
                let _ = app_handle.emit("vault-changed", payload);
            }
        }
    });

    let watcher_lock = WATCHER.get_or_init(|| Mutex::new(None));
    let mut guard = watcher_lock.lock().map_err(|_| "watcher lock")?;
    *guard = Some(watcher);
    Ok(())
}

/// Stop the active watcher, if any.
pub async fn stop_watcher() -> Result<(), String> {
    let watcher_lock = WATCHER.get_or_init(|| Mutex::new(None));
    let mut guard = watcher_lock.lock().map_err(|_| "watcher lock")?;
    *guard = None;
    Ok(())
}
