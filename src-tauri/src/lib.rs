use std::time::Duration;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
async fn scrape_in_webview(app: tauri::AppHandle, url: String) -> Result<(), String> {
    println!("scrape_in_webview called with URL: {}", url);

    let label = "scraper";

    // Create or reuse the hidden webview
    let win = if let Some(w) = app.get_webview_window(label) {
        w
    } else {
        WebviewWindowBuilder::new(&app, label, WebviewUrl::App("blank.html".into()))
            .visible(false)
            .title("Scraper")
            .build()
            .map_err(|e| e.to_string())?
    };

    // Navigate to the target URL
    win.eval(&format!("window.location.replace({:?});", url))
        .map_err(|e| e.to_string())?;

    // Wait for page to load and then poll for data
    let ah = app.clone();
    let win_label = win.label().to_string();

    tauri::async_runtime::spawn(async move {
        // Wait a bit for navigation to start
        tokio::time::sleep(Duration::from_millis(2000)).await;

        if let Some(w) = ah.get_webview_window(&win_label) {
            let _ = w.eval(r#"
                window.__TAURI__.core.invoke("handle_scrape_result", {
                    html: document.documentElement.outerHTML,
                });
            "#);
        } else {
            println!("Webview window not found during polling");
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    });

    Ok(())
}

#[tauri::command]
async fn handle_scrape_result(app: tauri::AppHandle, html: String) -> Result<(), String> {
    let _ = app.emit("scraper:result", serde_json::json!({
        "html": html,
    }));
    Ok(())
}

#[tauri::command]
async fn handle_scrape_error(app: tauri::AppHandle, error: String) -> Result<(), String> {
    println!("handle_scrape_error called with error: {}", error);
    let _ = app.emit("scraper:error", error);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scrape_in_webview,
            handle_scrape_result,
            handle_scrape_error,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
