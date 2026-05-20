pub const APP_CARGO_TOML: &str = r#"[package]
name = "app-template"
version = "0.1.0"
edition = "2021"

[dependencies]
wry = "0.24"
tao = "0.16"

[build-dependencies]
winres = "0.1"
"#;

pub const APP_BUILD_RS: &str = r#"fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("app.ico").exists() {
            res.set_icon("app.ico");
        }
        res.compile().unwrap();
    }
}
"#;

pub const APP_MAIN_RS: &str = r##"#![windows_subsystem = "windows"]

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();
    
    let window = WindowBuilder::new()
        .with_title(config::APP_NAME)
        .with_inner_size(tao::dpi::LogicalSize::new(config::APP_WIDTH, config::APP_HEIGHT))
        .build(&event_loop)?;

    if let Some(monitor) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size();
        let x = (screen_size.width as i32 - window_size.width as i32) / 2;
        let y = (screen_size.height as i32 - window_size.height as i32) / 2;
        window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
    }

    let webview = WebViewBuilder::new(window)?
        .with_user_agent(config::USER_AGENT)
        .with_devtools(true)
        .with_ipc_handler(|window, msg| {
            if msg == "fullscreen:true" {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            } else if msg == "fullscreen:false" {
                window.set_fullscreen(None);
            }
        })
        .with_initialization_script(r#"
            window.addEventListener('keydown', (e) => {
                if (e.altKey && e.key === 'ArrowLeft') {
                    window.history.back();
                }
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        let _ = &webview;

        match event {
            Event::NewEvents(StartCause::Init) => {}
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
"##;

pub const INSTALLER_CARGO_TOML: &str = r#"[package]
name = "installer-template"
version = "0.1.0"
edition = "2021"

[dependencies]
winreg = "0.50"

[build-dependencies]
winres = "0.1"
"#;

pub const INSTALLER_BUILD_RS: &str = r##"fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set_manifest(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#);
        res.compile().unwrap();
    }
}
"##;

pub const INSTALLER_MAIN_RS: &str = r#"use std::fs;
use std::path::PathBuf;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

mod config;

static APP_BINARY: &[u8] = include_bytes!("app_bin.exe");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let is_uninstall = args.iter().any(|arg| arg == "--uninstall");

    if is_uninstall {
        run_uninstaller()?;
    } else {
        run_installer()?;
    }

    Ok(())
}

fn run_installer() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("   Welcome to the {} Installer", config::APP_NAME);
    println!("==================================================");
    println!("Version: {}", config::APP_VERSION);
    println!("Publisher: {}", config::PUBLISHER);
    println!();

    println!("Press [ENTER] to start installation or Ctrl+C to cancel...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("Installing...");

    // 1. Get paths
    let local_app_data = std::env::var("LOCALAPPDATA")?;
    let install_dir = PathBuf::from(&local_app_data).join("Programs").join(config::APP_NAME);
    let app_exe_path = install_dir.join(format!("{}.exe", config::APP_NAME));
    let uninstall_exe_path = install_dir.join("uninstall.exe");

    // 2. Create installation directory
    fs::create_dir_all(&install_dir)?;

    // 3. Write application binary
    println!("Writing application files...");
    fs::write(&app_exe_path, APP_BINARY)?;

    // 4. Write icon if we have one compiled (we can copy our own icon if available)
    // Wait, the installer itself contains the icon compiled in, but we can also extract a file if needed.
    // However, the powershell script shortcut can point directly to the app_exe_path since the app exe
    // has the embedded icon compiled in via winres! This is awesome.
    
    // 5. Copy current running executable (ourselves) to the install directory as uninstall.exe
    let current_exe = std::env::current_exe()?;
    fs::copy(&current_exe, &uninstall_exe_path)?;

    // 6. Create shortcuts
    println!("Creating shortcuts...");
    let desktop_shortcut = get_desktop_path().join(format!("{}.lnk", config::APP_NAME));
    let start_menu_shortcut = get_start_menu_programs_path().join(format!("{}.lnk", config::APP_NAME));

    if let Some(parent) = desktop_shortcut.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = start_menu_shortcut.parent() {
        fs::create_dir_all(parent)?;
    }

    create_shortcut(
        app_exe_path.to_str().unwrap(),
        desktop_shortcut.to_str().unwrap(),
        app_exe_path.to_str().unwrap(),
    )?;

    create_shortcut(
        app_exe_path.to_str().unwrap(),
        start_menu_shortcut.to_str().unwrap(),
        app_exe_path.to_str().unwrap(),
    )?;

    // 7. Write registry entries for Windows Uninstall
    println!("Registering uninstaller...");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _disp) = hkcu.create_subkey(format!(
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}",
        config::APP_NAME
    ))?;

    key.set_value("DisplayName", &config::APP_NAME)?;
    key.set_value("DisplayVersion", &config::APP_VERSION)?;
    key.set_value("Publisher", &config::PUBLISHER)?;
    key.set_value("DisplayIcon", &app_exe_path.to_str().unwrap_or(""))?;
    key.set_value("UninstallString", &format!("\"{}\" --uninstall", uninstall_exe_path.to_str().unwrap_or("")))?;
    key.set_value("InstallLocation", &install_dir.to_str().unwrap_or(""))?;
    
    let dword_val: u32 = 1;
    key.set_value("NoModify", &dword_val)?;
    key.set_value("NoRepair", &dword_val)?;

    println!("\nInstallation completed successfully!");
    println!("Installed at: {}", install_dir.display());
    println!();
    println!("Press [ENTER] to launch {} and exit...", config::APP_NAME);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Launch the app
    Command::new(&app_exe_path).spawn()?;

    Ok(())
}

fn run_uninstaller() -> Result<(), Box<dyn std::error::Error>> {
    println!("==================================================");
    println!("   Welcome to the {} Uninstaller", config::APP_NAME);
    println!("==================================================");
    println!();
    
    println!("Are you sure you want to uninstall {}? [y/N]: ", config::APP_NAME);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    println!("Uninstalling...");

    // 1. Get paths
    let local_app_data = std::env::var("LOCALAPPDATA")?;
    let install_dir = PathBuf::from(&local_app_data).join("Programs").join(config::APP_NAME);
    
    // 2. Delete shortcuts
    let desktop_shortcut = get_desktop_path().join(format!("{}.lnk", config::APP_NAME));
    let start_menu_shortcut = get_start_menu_programs_path().join(format!("{}.lnk", config::APP_NAME));

    if desktop_shortcut.exists() {
        let _ = fs::remove_file(&desktop_shortcut);
    }
    if start_menu_shortcut.exists() {
        let _ = fs::remove_file(&start_menu_shortcut);
    }

    // 3. Delete registry key
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let uninstall_key_path = format!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}", config::APP_NAME);
    let _ = hkcu.delete_subkey_all(&uninstall_key_path);

    // 4. Self deletion trick.
    // We will spawn a detached cmd process that waits a second, then deletes the install directory.
    let install_dir_str = install_dir.to_str().unwrap().to_string();
    println!("Cleaning up files...");
    
    // Start cmd in background to delete the folder
    Command::new("cmd")
        .args(&[
            "/c",
            &format!("timeout /t 1 && rmdir /s /q \"{}\"", install_dir_str)
        ])
        .spawn()?;

    println!("Uninstall completed. This window will now close.");
    
    Ok(())
}

fn create_shortcut(target_path: &str, shortcut_path: &str, icon_path: &str) -> std::io::Result<()> {
    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut('{}'); \
         $Shortcut.TargetPath = '{}'; \
         $Shortcut.IconLocation = '{}'; \
         $Shortcut.Save();",
        shortcut_path.replace("'", "''"),
        target_path.replace("'", "''"),
        icon_path.replace("'", "''")
    );
    
    let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command", &script])
        .output()?;
        
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(())
}

fn expand_env_string(raw: &str) -> String {
    let mut result = raw.to_string();
    if result.contains("%USERPROFILE%") {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            result = result.replace("%USERPROFILE%", &user_profile);
        }
    }
    if result.contains("%APPDATA%") {
        if let Ok(app_data) = std::env::var("APPDATA") {
            result = result.replace("%APPDATA%", &app_data);
        }
    }
    result
}

fn get_desktop_path() -> PathBuf {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders") {
        if let Ok(raw_val) = key.get_value::<String, _>("Desktop") {
            let expanded = expand_env_string(&raw_val);
            let path = PathBuf::from(expanded);
            if path.exists() {
                return path;
            }
        }
    }
    // Fallback to default
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(user_profile).join("Desktop");
    }
    PathBuf::from(".")
}

fn get_start_menu_programs_path() -> PathBuf {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders") {
        if let Ok(raw_val) = key.get_value::<String, _>("Programs") {
            let expanded = expand_env_string(&raw_val);
            let path = PathBuf::from(expanded);
            if path.exists() {
                return path;
            }
        }
    }
    // Fallback to default
    if let Ok(app_data) = std::env::var("APPDATA") {
        return PathBuf::from(app_data).join("Microsoft").join("Windows").join("Start Menu").join("Programs");
    }
    PathBuf::from(".")
}
"#;
