#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use henkei_runtime::config::AppConfig;
use henkei_runtime::{ipc, navigation, window};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};
use wry::{NewWindowResponse, WebContext, WebViewBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        // Fix for WebKitGTK GStreamer HTML5 video playback stalls on Linux (Wayland/Mesa/NVIDIA)
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let mut config = AppConfig::load();

    // CLI Arguments Handling
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("{} (Henkei Desktop App)", config.name);
                println!("\nUsage: [OPTIONS]");
                println!("\nOptions:");
                println!("  -h, --help       Print help information");
                println!("  -v, --version    Print version");
                println!("  --url <URL>      Override initial application URL");
                println!("  --devtools       Enable web developer tools");
                #[cfg(target_os = "linux")]
                println!("  --install        Install desktop launcher to ~/.local/share/applications");
                return Ok(());
            }
            "--version" | "-v" => {
                println!("{} 1.0.0 (Henkei v0.2.0)", config.name);
                return Ok(());
            }
            #[cfg(target_os = "linux")]
            "--install" => {
                let exe_path = std::env::current_exe()?;
                let sanitized: String = config.name.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                if let Some(data_dir) = dirs::data_dir() {
                    let apps_dir = data_dir.join("applications");
                    std::fs::create_dir_all(&apps_dir)?;
                    let desktop_content = format!(
                        "[Desktop Entry]\n\
                         Type=Application\n\
                         Name={}\n\
                         Exec=\"{}\"\n\
                         Terminal=false\n\
                         Categories=Network;WebBrowser;\n\
                         Comment=Native desktop webapp powered by Henkei\n",
                        config.name,
                        exe_path.display()
                    );
                    let desktop_path = apps_dir.join(format!("{}.desktop", sanitized));
                    std::fs::write(&desktop_path, desktop_content)?;
                    println!("✔ Successfully installed {} desktop shortcut to {}!", config.name, desktop_path.display());
                }
                return Ok(());
            }
            "--url" => {
                if i + 1 < args.len() {
                    config.url = args[i + 1].clone();
                    i += 1;
                }
            }
            "--devtools" => {
                config.devtools = true;
            }
            _ => ()
        }
        i += 1;
    }

    let data_dir = config.get_app_data_dir();
    let webview_cache_dir = data_dir.join("webview");
    let _ = std::fs::create_dir_all(&webview_cache_dir);

    let event_loop = EventLoop::new();

    let window = Arc::new(
        WindowBuilder::new()
            .with_title(&config.name)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .build(&event_loop)?
    );

    // Apply saved geometry or smart center
    window::apply_initial_placement(
        &window,
        &data_dir,
        config.width,
        config.height,
        config.persist_geometry,
    );

    let mut web_context = WebContext::new(Some(webview_cache_dir));

    let root_url = config.url.clone();
    let allowed_domains = config.allowed_domains.clone();

    let root_url_for_popup = root_url.clone();
    let allowed_for_popup = allowed_domains.clone();

    let root_url_for_ipc = root_url.clone();
    let allowed_for_ipc = allowed_domains.clone();

    let is_fullscreen = Arc::new(AtomicBool::new(false));
    let is_fullscreen_clone = is_fullscreen.clone();

    let init_script = ipc::build_initialization_script(
        config.inject_css.as_deref(),
        config.inject_js.as_deref(),
    );

    let builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_url(&root_url)
        .with_user_agent(&config.user_agent)
        .with_devtools(config.devtools)
        .with_initialization_script(&init_script)
        .with_new_window_req_handler(move |url: String, _features| {
            if navigation::should_navigate_internally(&url, &root_url_for_popup, &allowed_for_popup) {
                NewWindowResponse::Allow
            } else {
                // Deny unrequested external popups and ad popups without spawning browser windows!
                NewWindowResponse::Deny
            }
        })
        .with_ipc_handler({
            let window_ref = window.clone();
            move |msg| {
                let body = msg.body();
                if body == "henkei:fullscreen:true" {
                    is_fullscreen_clone.store(true, Ordering::SeqCst);
                    window_ref.set_fullscreen(Some(Fullscreen::Borderless(None)));
                } else if body == "henkei:fullscreen:false" {
                    is_fullscreen_clone.store(false, Ordering::SeqCst);
                    window_ref.set_fullscreen(None);
                } else if let Some(title) = body.strip_prefix("henkei:title:") {
                    if !title.trim().is_empty() {
                        window_ref.set_title(title);
                    }
                } else if let Some(ext_url) = body.strip_prefix("henkei:open_external:") {
                    if navigation::should_open_in_browser(ext_url, &root_url_for_ipc, &allowed_for_ipc) {
                        let _ = open::that_detached(ext_url);
                    }
                }
            }
        });

    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().ok_or("Failed to get default_vbox on window")?;
        builder.build_gtk(vbox)?
    };

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window)?;

    let data_dir_clone = data_dir.clone();
    let persist = config.persist_geometry;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let _ = &webview;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if persist {
                    window::save_geometry(&window, &data_dir_clone);
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::LoopDestroyed => {
                if persist {
                    window::save_geometry(&window, &data_dir_clone);
                }
            }
            _ => (),
        }
    });
}
