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
    // 仅允许 http/https, 且禁止 URL 内嵌账号密码(防凭据注入/异常协议)
    let mut parsed = reqwest::Url::parse(&url).map_err(|e| format!("无效地址: {e}"))?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("仅支持 http/https 地址".to_string()),
    }
    if parsed.host_str().is_none() {
        return Err("地址缺少主机名".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("地址中不允许携带内嵌账号密码".to_string());
    }

    // 超时上限 30 秒, 防止 webview 触发无限等待
    let timeout_ms = timeout_ms.min(30_000);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| e.to_string())?;

    // 追加 password 查询参数(DontCrack 鉴权约定)
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            parsed.query_pairs_mut().append_pair("password", &pwd);
        }
    }

    let mut resp = match method.to_uppercase().as_str() {
        "POST" => client.post(parsed).send().await.map_err(|e| e.to_string())?,
        _ => client.get(parsed).send().await.map_err(|e| e.to_string())?,
    };

    let status = resp.status();
    // 响应体上限 16MB, 防止异常 agent 返回超大响应撑爆桌面端内存。
    // 必须在流式读取过程中限流: 先 .text() 全量读入后再检查长度, 内存在检查前
    // 已被耗尽, 上限形同虚设。
    const MAX_BODY: u64 = 16 * 1024 * 1024;
    if let Some(len) = resp.content_length() {
        if len > MAX_BODY {
            return Err("响应体超过 16MB 上限".to_string());
        }
    }
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        if buf.len() as u64 + chunk.len() as u64 > MAX_BODY {
            return Err("响应体超过 16MB 上限".to_string());
        }
        buf.extend_from_slice(&chunk);
    }
    let body = String::from_utf8(buf).map_err(|e| format!("响应体不是合法 UTF-8: {e}"))?;
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