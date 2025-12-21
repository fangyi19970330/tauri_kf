// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    } catch (e) {
        console.error('inject listener failed', e);
    }
})();"#;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![download_image])
        .on_page_load(|window, _| {
            let _ = window.eval(INJECT_IMAGE_DOWNLOAD_LISTENER);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
