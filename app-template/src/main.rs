#![windows_subsystem = "windows"]

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    
    // Create the window
    let window = WindowBuilder::new()
        .with_title(config::APP_NAME)
        .with_inner_size(tao::dpi::LogicalSize::new(config::APP_WIDTH, config::APP_HEIGHT))
        .build(&event_loop)?;

    // Center window
    if let Some(monitor) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size();
        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
        let y = (screen_size.height as i32 - window_size.height as i32) / 2;
        window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
    }

    // Build the WebView
    let webview = WebViewBuilder::new(window)?
        .with_user_agent(config::USER_AGENT)
        // Enable devtools in case the user wants to debug or inspect (e.g. via F12)
        .with_devtools(true)
        .with_ipc_handler(|window, msg| {
            if msg == "fullscreen:true" {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            } else if msg == "fullscreen:false" {
                window.set_fullscreen(None);
            }
        })
        // Inject custom JS for keyboard navigation helpers and fullscreen handling
        .with_initialization_script(r#"
            window.addEventListener('keydown', (e) => {
                // Alt + ArrowLeft -> Back
                if (e.altKey && e.key === 'ArrowLeft') {
                    window.history.back();
                }
                // Alt + ArrowRight -> Forward
                if (e.altKey && e.key === 'ArrowRight') {
                    window.history.forward();
                }
            });
            document.addEventListener('fullscreenchange', () => {
                if (document.fullscreenElement) {
                    window.ipc.postMessage('fullscreen:true');
                } else {
                    window.ipc.postMessage('fullscreen:false');
                }
            });
            document.addEventListener('webkitfullscreenchange', () => {
                if (document.webkitFullscreenElement) {
                    window.ipc.postMessage('fullscreen:true');
                } else {
                    window.ipc.postMessage('fullscreen:false');
                }
            });
        "#)
        .with_url(config::APP_URL)?
        .build()?;

    // Event Loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        // Keep webview alive for the duration of the event loop
        let _ = &webview;

        match event {
            Event::NewEvents(StartCause::Init) => {
                // Log initialization if in debug
                #[cfg(debug_assertions)]
                println!("App started, loading {}", config::APP_URL);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}
