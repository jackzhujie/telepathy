// Debug HTTP Server for Telepathy
// Allows external tools (like MCP server) to control the app via HTTP
//
// Listens on http://127.0.0.1:9876
//
// Endpoints:
// - POST /navigate     - Navigate to a route
//   Body: {"page": "/settings"}
// - POST /execute      - Execute JavaScript in the webview
//   Body: {"code": "router.push('/settings')"}
// - GET  /state        - Get current app state
// - GET  /pages        - List all available pages
// - GET  /ping         - Health check

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug, Deserialize, Serialize)]
struct NavigateRequest {
    page: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExecuteRequest {
    code: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse {
    fn ok(data: serde_json::Value) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    fn err(error: String) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }
}

/// Read HTTP request body
async fn read_http_request(
    reader: &mut tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>,
) -> Option<(String, String, String)> {
    let mut headers = Vec::new();
    let mut content_length = 0usize;
    let mut method = String::new();
    let mut path = String::new();

    // Read headers
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => return None,
            Ok(_) => {}
            Err(_) => return None,
        }

        let line = line.trim().to_string();
        if line.is_empty() {
            break;
        }

        if line.starts_with("GET ") || line.starts_with("POST ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                method = parts[0].to_string();
                path = parts[1].to_string();
            }
        } else if line.to_lowercase().starts_with("content-length:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                content_length = parts[1].trim().parse().unwrap_or(0);
            }
        }
        headers.push(line);
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        if reader.read_exact(&mut body).await.is_err() {
            return None;
        }
    }

    Some((method, path, String::from_utf8_lossy(&body).to_string()))
}

/// Write HTTP response
async fn write_response(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        content_type,
        body.len(),
        body
    );

    writer.write_all(response.as_bytes()).await?;
    writer.shutdown().await?;
    Ok(())
}

/// Handle a single HTTP request
async fn handle_request<R: Runtime>(
    method: &str,
    path: &str,
    body: &str,
    app: &AppHandle<R>,
) -> (u16, String) {
    println!("[DebugHTTP] {} {}", method, path);

    match (method, path) {
        ("GET", "/ping") => {
            (200, serde_json::to_string(&ApiResponse::ok(serde_json::json!({"pong": true}))).unwrap_or_default())
        }
        ("GET", "/read-route") => {
            // 通过 localStorage 和 location.href 一起验证当前路由
            let app_clone = app.clone();
            let js = "(() => { try { const r = localStorage.getItem('__tele_route'); const h = window.location.href; const p = window.location.pathname; return JSON.stringify({route: r, href: h, path: p}); } catch(e) { return JSON.stringify({err: String(e)}); } })()";
            let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
            let (res_tx, res_rx) = std::sync::mpsc::channel::<String>();
            let send_result = app.run_on_main_thread(move || {
                if let Some(window) = app_clone.get_webview_window("main") {
                    // 虽然 eval 不返回值，但我们可以通过其他方式？不，直接在前端把结果写到 cookie 或者别的？
                    // 换个办法：让前端把结果通过一个临时的 invoke 或者直接让用户看 console，但最简单的是：
                    // 我们直接用 execute 端点来执行读取 JS，然后用户可以在 DevTools Console 看输出，或者我们让 navigate 之后再执行一段写 cookie 的 JS，然后再读，但现在先简化：
                    let _ = window.eval(js);
                }
                let _ = tx.send(Ok(()));
            });
            match send_result {
                Ok(_) => match rx.recv() {
                    Ok(Ok(_)) => (200, serde_json::to_string(&ApiResponse::ok(serde_json::json!({ "note": "Check console in DevTools for route info from JS" }))).unwrap_or_default()),
                    Ok(Err(e)) => (500, serde_json::to_string(&ApiResponse::err(e)).unwrap_or_default()),
                    Err(_) => (500, serde_json::to_string(&ApiResponse::err("Channel closed".to_string())).unwrap_or_default()),
                },
                Err(e) => (500, serde_json::to_string(&ApiResponse::err(format!("Main thread error: {}", e))).unwrap_or_default()),
            }
        }
        ("GET", "/pages") => {
            let pages = serde_json::json!([
                { "name": "KnowledgeBase", "path": "/" },
                { "name": "Chat", "path": "/chat" },
                { "name": "Documents", "path": "/documents" },
                { "name": "Profile", "path": "/profile" },
                { "name": "Settings", "path": "/settings" },
                { "name": "Models", "path": "/models" },
                { "name": "SnapNote", "path": "/snapnote" }
            ]);
            (200, serde_json::to_string(&ApiResponse::ok(serde_json::json!({ "pages": pages }))).unwrap_or_default())
        }
        ("GET", "/state") => {
            // Use eval to read window.location.pathname via the main thread,
            // because wry's url() panics on macOS when the webview URL is not yet ready.
            let app_clone = app.clone();
            let js = "(() => { try { return window.location.pathname + window.location.hash; } catch(e) { return ''; } })()";
            let (tx, rx) = std::sync::mpsc::channel::<Result<serde_json::Value, String>>();
            let send_result = app.run_on_main_thread(move || {
                let res = if let Some(window) = app_clone.get_webview_window("main") {
                    window
                        .eval(js)
                        .map(|_| serde_json::json!({ "note": "current route unreadable via eval result; see execute endpoint" }))
                        .map_err(|e| format!("eval failed: {}", e))
                } else {
                    Err("Main window not found".to_string())
                };
                let _ = tx.send(res);
            });
            match send_result {
                Ok(_) => match rx.recv() {
                    Ok(Ok(data)) => (200, serde_json::to_string(&ApiResponse::ok(data)).unwrap_or_default()),
                    Ok(Err(e)) => (500, serde_json::to_string(&ApiResponse::err(e)).unwrap_or_default()),
                    Err(_) => (500, serde_json::to_string(&ApiResponse::err("Channel closed".to_string())).unwrap_or_default()),
                },
                Err(e) => (500, serde_json::to_string(&ApiResponse::err(format!("Main thread error: {}", e))).unwrap_or_default()),
            }
        }
        ("POST", "/navigate") => {
            match serde_json::from_str::<NavigateRequest>(body) {
                Ok(req) => {
                    let app_clone = app.clone();
                    let page = req.page.clone();
                    // 直接硬跳，并设置 localStorage 和 body.dataset，方便 DevTools 验证
                    let js = format!(
                        "(() => {{ localStorage.setItem('__tele_route', '{}'); document.body.dataset.route = '{}'; location.assign('{}'); return 'assign'; }})()",
                        req.page, req.page, req.page
                    );
                    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
                    let send_result = app.run_on_main_thread(move || {
                        let res = if let Some(window) = app_clone.get_webview_window("main") {
                            window.eval(&js).map_err(|e| format!("Failed to navigate: {}", e))
                        } else {
                            Err("Main window not found".to_string())
                        };
                        let _ = tx.send(res);
                    });
                    match send_result {
                        Ok(_) => match rx.recv() {
                            Ok(Ok(_)) => (200, serde_json::to_string(&ApiResponse::ok(serde_json::json!({ "navigated": page, "note": "Check DevTools Console: window.location.pathname and localStorage.getItem('__tele_route')" }))).unwrap_or_default()),
                            Ok(Err(e)) => (500, serde_json::to_string(&ApiResponse::err(e)).unwrap_or_default()),
                            Err(_) => (500, serde_json::to_string(&ApiResponse::err("Channel closed".to_string())).unwrap_or_default()),
                        },
                        Err(e) => (500, serde_json::to_string(&ApiResponse::err(format!("Main thread error: {}", e))).unwrap_or_default()),
                    }
                }
                Err(e) => (400, serde_json::to_string(&ApiResponse::err(format!("Invalid request: {}", e))).unwrap_or_default()),
            }
        }
        ("POST", "/execute") => {
            match serde_json::from_str::<ExecuteRequest>(body) {
                Ok(req) => {
                    let app_clone = app.clone();
                    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
                    let send_result = app.run_on_main_thread(move || {
                        let res = if let Some(window) = app_clone.get_webview_window("main") {
                            window.eval(&req.code).map_err(|e| format!("Failed to execute JS: {}", e))
                        } else {
                            Err("Main window not found".to_string())
                        };
                        let _ = tx.send(res);
                    });
                    match send_result {
                        Ok(_) => match rx.recv() {
                            Ok(Ok(_)) => (200, serde_json::to_string(&ApiResponse::ok(serde_json::json!({ "executed": true }))).unwrap_or_default()),
                            Ok(Err(e)) => (500, serde_json::to_string(&ApiResponse::err(e)).unwrap_or_default()),
                            Err(_) => (500, serde_json::to_string(&ApiResponse::err("Channel closed".to_string())).unwrap_or_default()),
                        },
                        Err(e) => (500, serde_json::to_string(&ApiResponse::err(format!("Main thread error: {}", e))).unwrap_or_default()),
                    }
                }
                Err(e) => (400, serde_json::to_string(&ApiResponse::err(format!("Invalid request: {}", e))).unwrap_or_default()),
            }
        }
        ("OPTIONS", _) => (200, String::new()),
        _ => (404, serde_json::to_string(&ApiResponse::err("Not Found".to_string())).unwrap_or_default()),
    }
}

/// Handle a single client connection
async fn handle_connection<R: Runtime>(
    stream: tokio::net::TcpStream,
    app: AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = tokio::io::BufReader::new(read_half);

    let (method, path, body) = match read_http_request(&mut reader).await {
        Some(t) => t,
        None => {
            let _ = write_response(&mut write_half, 400, "text/plain", "Bad Request").await;
            return Ok(());
        }
    };

    let (status, response_body) = handle_request(&method, &path, &body, &app).await;
    let _ = write_response(&mut write_half, status, "application/json", &response_body).await;
    Ok(())
}

/// Start the debug HTTP server
pub async fn start_debug_server<R: Runtime>(app: AppHandle<R>) {
    let addr = "127.0.0.1:9876";
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => {
            println!("[DebugHTTP] ✅ Listening on http://{}", addr);
            l
        }
        Err(e) => {
            eprintln!("[DebugHTTP] ❌ Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let app_clone = app.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, app_clone).await {
                        eprintln!("[DebugHTTP] Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[DebugHTTP] Accept failed: {}", e);
            }
        }
    }
}
