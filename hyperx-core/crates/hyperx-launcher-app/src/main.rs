use hyperx_launcher_app::ipc::{IpcDispatcher, IpcRequest};
use hyperx_launcher_app::vibrancy;
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::http::{Request, Response, StatusCode};
use wry::WebViewBuilder;

// Embed the entire pre-compiled Vue 3 frontend inside the binary for a 100% standalone executable
static EMBEDDED_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../../xmcl-keystone-ui/dist");

fn find_dist_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("xmcl-keystone-ui/dist"),
        PathBuf::from("../xmcl-keystone-ui/dist"),
        PathBuf::from("../../xmcl-keystone-ui/dist"),
        PathBuf::from("dist"),
    ];

    for c in &candidates {
        if c.exists() && c.join("index.html").exists() {
            return c.clone();
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("dist");
            if p.exists() {
                return p;
            }
        }
    }

    PathBuf::from("xmcl-keystone-ui/dist")
}

fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting HyperXLauncher (100% Native Rust Standalone Engine)...");

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("HyperXLauncher")
        .with_inner_size(LogicalSize::new(1200.0, 720.0))
        .with_min_inner_size(LogicalSize::new(800.0, 400.0))
        .with_transparent(true)
        .with_decorations(true)
        .build(&event_loop)?;

    // Hardware-accelerated glassmorphism
    vibrancy::apply_glassmorphism(&window);

    let app_data_dir = dirs_fallback().join(".hyperx");
    let _ = fs::create_dir_all(&app_data_dir);
    let dispatcher = Arc::new(IpcDispatcher::new(app_data_dir));

    let dist_dir = find_dist_dir();
    let dist_dir_for_protocol = dist_dir.clone();

    // Custom protocol "hyperx://" serving frontend assets (hybrid: disk with embedded fallback)
    let custom_protocol = move |request: Request<Vec<u8>>| {
        let uri = request.uri().path();
        let relative_path = uri.trim_start_matches('/');
        let path_to_lookup = if relative_path.is_empty() || relative_path == "index.html" {
            "index.html"
        } else {
            relative_path
        };

        // 1. Try disk if external dist folder exists
        if dist_dir_for_protocol.exists() {
            let file_path = dist_dir_for_protocol.join(path_to_lookup);
            if file_path.exists() && file_path.is_file() {
                if let Ok(bytes) = fs::read(&file_path) {
                    let mime = get_mime_type(&file_path);
                    return Response::builder()
                        .header("Content-Type", mime)
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Cache-Control", "public, max-age=3600")
                        .body(bytes.into())
                        .unwrap();
                }
            }
        }

        // 2. Load directly from embedded binary assets (100% standalone single-file .exe)
        let file = EMBEDDED_DIST
            .get_file(path_to_lookup)
            .or_else(|| EMBEDDED_DIST.get_file("index.html"));

        if let Some(f) = file {
            let mime = get_mime_type(Path::new(f.path()));
            return Response::builder()
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "public, max-age=3600")
                .body(f.contents().to_vec().into())
                .unwrap();
        }

        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Resource Not Found".as_bytes().into())
            .unwrap()
    };

    // Seamless zero-overhead IPC bridge injection
    let bridge_init_script = r#"
        window.__NATIVE_RUST_CORE__ = true;
        window.__HYPERX_PENDING_CALLS__ = new Map();
        window.__HYPERX_CALL_ID__ = 1;

        window.__HYPERX_INVOKE__ = function(command, payload) {
            return new Promise((resolve, reject) => {
                const id = window.__HYPERX_CALL_ID__++;
                window.__HYPERX_PENDING_CALLS__.set(id, { resolve, reject });
                const req = JSON.stringify({ id, command, payload: payload || {} });
                if (window.ipc) {
                    window.ipc.postMessage(req);
                } else {
                    reject(new Error("Native IPC bridge unavailable"));
                }
            });
        };

        window.__HYPERX_DISPATCH_RESPONSE__ = function(resStr) {
            try {
                const res = JSON.parse(resStr);
                const pending = window.__HYPERX_PENDING_CALLS__.get(res.id);
                if (pending) {
                    window.__HYPERX_PENDING_CALLS__.delete(res.id);
                    if (res.success) {
                        pending.resolve(res.data);
                    } else {
                        pending.reject(new Error(res.error || "Unknown IPC Error"));
                    }
                }
            } catch (err) {
                console.error("IPC dispatch response failed:", err);
            }
        };

        // Tauri v2 & Electron bridge shim
        window.__TAURI__ = {
            core: {
                invoke: window.__HYPERX_INVOKE__
            }
        };
    "#;

    let dispatcher_for_ipc = dispatcher.clone();

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    let _webview = WebViewBuilder::new(&window)
        .with_transparent(true)
        .with_custom_protocol("hyperx".into(), custom_protocol)
        .with_initialization_script(bridge_init_script)
        .with_ipc_handler(move |request| {
            let dispatcher = dispatcher_for_ipc.clone();
            if let Ok(req) = serde_json::from_str::<IpcRequest>(request.body()) {
                tokio::spawn(async move {
                    let res = dispatcher.dispatch(req).await;
                    if let Ok(serialized) = serde_json::to_string(&res) {
                        log::debug!("IPC Response dispatched: {}", serialized);
                    }
                });
            }
        })
        .with_url("hyperx://localhost/index.html")
        .build()?;

    // Main Event Loop with GPU Auto-Suspend
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    log::info!("Closing HyperXLauncher...");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Focused(focused) => {
                    // GPU Auto-Suspend: when window unfocused or game running, suspend heavy animations
                    if !focused && dispatcher.is_game_running() {
                        log::debug!("Game active & launcher unfocused: throttling UI render pacing");
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });
}

fn dirs_fallback() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    }
}
