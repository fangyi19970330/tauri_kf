// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::PathBuf;
use tauri::Manager;

static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tauri::command]
async fn download_image(url: String, path: String) -> Result<(), String> {
        let parsed_url = url::Url::parse(&url).map_err(|e| format!("invalid url: {e}"))?;
        match parsed_url.scheme() {
                "http" | "https" => {}
                other => return Err(format!("unsupported url scheme: {other}")),
        }

        let response = reqwest::get(parsed_url)
                .await
                .map_err(|e| format!("request failed: {e}"))?;
        let status = response.status();
        if !status.is_success() {
                return Err(format!("request failed with status: {status}"));
        }

        let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("failed to read response body: {e}"))?;

        let parent = std::path::Path::new(&path)
                .parent()
                .map(|p| p.to_path_buf());
        if let Some(parent) = parent {
                if !parent.as_os_str().is_empty() {
                        tokio::fs::create_dir_all(&parent)
                                .await
                                .map_err(|e| format!("failed to create parent directory: {e}"))?;
                }
        }

        tokio::fs::write(&path, &bytes)
                .await
                .map_err(|e| format!("failed to write file: {e}"))?;

        Ok(())
}

#[tauri::command]
fn open_new_window(app: tauri::AppHandle, url: Option<String>) -> Result<(), String> {
    let label = format!("win-{}", WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed));

    let window_url = match url {
        Some(u) if !u.trim().is_empty() => {
            let parsed = url::Url::parse(&u).map_err(|e| format!("invalid url: {e}"))?;
            // Basic safety: only allow opening the configured domain.
            if parsed.host_str() != Some("rulai.dqpfgcl.cn") {
                return Err("blocked: unsupported domain".to_string());
            }
            tauri::WindowUrl::External(parsed)
        }
        _ => tauri::WindowUrl::External(
            url::Url::parse("https://guanyu.a-c2.cn/").map_err(|e| e.to_string())?,
        ),
    };

    tauri::WindowBuilder::new(&app, label, window_url)
        .title("关羽2")
        .inner_size(775.0, 800.0)
        .center()
        .build()
        .map(|_| ())
        .map_err(|e| format!("failed to create window: {e}"))
}

const INJECT_IMAGE_DOWNLOAD_LISTENER: &str = r#"(function () {
    try {
        const dialog = window.__TAURI__ && window.__TAURI__.dialog;
        const invoke = window.__TAURI__ && (window.__TAURI__.invoke || (window.__TAURI__.tauri && window.__TAURI__.tauri.invoke));

        if (!dialog || !invoke) return;
        if (window.__IMG_DOWNLOAD_LISTENER_INSTALLED__) return;
        window.__IMG_DOWNLOAD_LISTENER_INSTALLED__ = true;

        function deriveFileName(url) {
            try {
                const u = new URL(url);
                const last = (u.pathname || '').split('/').filter(Boolean).pop() || 'image';
                if (last.includes('.')) return last;
                return last + '.png';
            } catch (_) {
                return 'image.png';
            }
        }

        document.addEventListener('click', async function (event) {
            const target = event.target;
            if (!(target instanceof HTMLImageElement)) return;

            const url = target.currentSrc || target.src;
            if (!url) return;

            event.preventDefault();
            event.stopPropagation();

            const defaultPath = deriveFileName(url);
            const savePath = await dialog.save({
                defaultPath,
                filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg'] }]
            });
            if (!savePath) return;

            try {
                await invoke('download_image', { url, path: savePath });
            } catch (e) {
                console.error('download_image failed', e);
                alert('下载失败：' + (e && (e.message || e)));
            }
        }, true);

        // Multi-window: Ctrl/Cmd + Shift + N to open a new window.
        document.addEventListener('keydown', async function (event) {
            const key = (event.key || '').toLowerCase();
            if (key !== 'n') return;
            if (!(event.ctrlKey || event.metaKey) || !event.shiftKey) return;

            const active = document.activeElement;
            if (active && (
                active.tagName === 'INPUT' ||
                active.tagName === 'TEXTAREA' ||
                active.isContentEditable
            )) {
                return;
            }

            event.preventDefault();
            try {
                await invoke('open_new_window', { url: window.location.href });
            } catch (e) {
                console.error('open_new_window failed', e);
            }
        }, true);
    } catch (e) {
        console.error('inject listener failed', e);
    }
})();"#;

fn arg_value(prefix: &str) -> Option<String> {
    std::env::args().find_map(|arg| arg.strip_prefix(prefix).map(|v| v.to_string()))
}

fn find_next_instance_number() -> u32 {
    let data_dir = PathBuf::from("data");
    if !data_dir.is_dir() {
        return 1;
    }

    let mut max_num = 0u32;
    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(num_str) = name.strip_prefix("instance-") {
                    if let Ok(num) = num_str.parse::<u32>() {
                        max_num = max_num.max(num);
                    }
                }
            }
        }
    }
    max_num + 1
}

fn resolve_user_data_dir_from_args() -> PathBuf {
    // Explicit user-data-dir has highest priority
    if let Some(dir) = arg_value("--user-data-dir=") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    // Named profile
    if let Some(profile) = arg_value("--profile=") {
        let profile = profile.trim();
        if !profile.is_empty() {
            return PathBuf::from("data").join(profile);
        }
    }

    // Auto-assign instance number
    let instance_num = find_next_instance_number();
    PathBuf::from("data").join(format!("instance-{}", instance_num))
}

fn apply_user_data_dir_from_args() {
    let dir = resolve_user_data_dir_from_args();

    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("failed to create user data dir {}: {e}", dir.display());
        return;
    }

    // Windows WebView2: isolate profile storage per process.
    #[cfg(target_os = "windows")]
    {
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &dir);
    }
}

fn main() {
    // Must happen before Tauri creates the WebView environment.
    apply_user_data_dir_from_args();

    tauri::Builder::default()
        .setup(|app| {
            let dir = resolve_user_data_dir_from_args();
            let _ = std::fs::create_dir_all(&dir);
            let _ = app.handle().fs_scope().allow_directory(dir, true);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![download_image, open_new_window])
        .on_page_load(|window, _| {
            let _ = window.eval(INJECT_IMAGE_DOWNLOAD_LISTENER);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
