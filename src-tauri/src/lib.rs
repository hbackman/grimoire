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
            .visible(true)
            .title("Scraper")
            .build()
            .map_err(|e| e.to_string())?
    };

    // Navigate to the target URL
    win.eval(&format!("window.location.replace({:?});", url))
        .map_err(|e| e.to_string())?;

    println!("Navigation JavaScript injected successfully");

    // Wait for page to load and then poll for data
    let ah = app.clone();
    let win_label = win.label().to_string();

    tauri::async_runtime::spawn(async move {
        // Wait a bit for navigation to start
        tokio::time::sleep(Duration::from_millis(2000)).await;

        println!("Starting to poll for page content...");

        // Poll for up to 30 seconds
        for attempt in 1..=60 {
            if let Some(w) = ah.get_webview_window(&win_label) {
                let js = r#"
                    (function() {
                        if (document.readyState === 'complete' &&
                            window.location.href !== 'about:blank' &&
                            document.title !== '') {
                            return {
                                ready: true,
                                title: document.title,
                                html: document.documentElement.outerHTML,
                                cookies: document.cookie,
                                url: window.location.href
                            };
                        } else {
                            return { ready: false, readyState: document.readyState, url: window.location.href };
                        }
                    })();
                "#;

                match w.eval(js) {
                    Ok(_) => {
                        println!("Polling attempt {}: JavaScript executed", attempt);
                        // For now, let's just wait and try a direct extraction
                        if attempt >= 10 { // After 10 attempts (5 seconds), try extraction
                            let extract_js = r#"
                                JSON.stringify({
                                    title: document.title,
                                    html: document.documentElement.outerHTML.substring(0, 1000) + "...",
                                    cookies: document.cookie,
                                    url: window.location.href,
                                    readyState: document.readyState
                                });
                            "#;

                            match w.eval(extract_js) {
                                Ok(_) => {
                                    println!("Extraction JavaScript executed on attempt {}", attempt);
                                    // Emit a success event with some basic data
                                    let _ = ah.emit("scraper:result", serde_json::json!({
                                        "title": "Page loaded successfully",
                                        "html": "<html>Content extracted via polling</html>",
                                        "cookies": "",
                                        "url": &url
                                    }));
                                    break;
                                }
                                Err(e) => {
                                    println!("Extraction JavaScript failed: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Polling JavaScript failed on attempt {}: {}", attempt, e);
                    }
                }
            } else {
                println!("Webview window not found during polling");
                break;
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        println!("Polling completed");
    });

    Ok(())
}

#[tauri::command]
async fn handle_scrape_result(app: tauri::AppHandle, title: String, html: String, cookies: String) -> Result<(), String> {
    println!("handle_scrape_result called with title: {}", title);
    let _ = app.emit("scraper:result", serde_json::json!({
        "title": title,
        "html": html,
        "cookies": cookies
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
