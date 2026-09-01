// ─────────────────────────────────────────────────────────────
// FasterEdge 开源项目
// Github: https://github.com/FasterEdge
// Gitee:  https://gitee.com/FasterEdge
// ─────────────────────────────────────────────────────────────
use tauri::Manager;

/// 向 DontCrack 服务端发起请求（GET/POST），返回响应体文本。
/// 通过 Rust 侧 reqwest 代理，规避 WebView 的 CORS 限制。
#[tauri::command]
async fn dc_request(
    url: String,
    method: String,
    password: Option<String>,
    timeout_ms: u64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| e.to_string())?;

    // 解析 URL 并追加 password 查询参数（DontCrack 鉴权约定）
    let mut parsed = reqwest::Url::parse(&url).map_err(|e| format!("无效地址: {e}"))?;
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            parsed.query_pairs_mut().append_pair("password", &pwd);
        }
    }

    let resp = match method.to_uppercase().as_str() {
        "POST" => client.post(parsed).send().await.map_err(|e| e.to_string())?,
        _ => client.get(parsed).send().await.map_err(|e| e.to_string())?,
    };

    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }
    Ok(body)
}

#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "DontCrack Desktop",
        "version": env!("CARGO_PKG_VERSION"),
        "api": "DontCrack HTTP (startup/heartbeat/shutdown)"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![dc_request, app_info])
        .setup(|app| { let _ = app.get_webview_window("main"); Ok(()) })
        .run(tauri::generate_context!())
        .expect("error while running DontCrack Desktop");
}