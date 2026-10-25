use clap::{Parser, ValueEnum};
use henkei_builder::TargetPlatform;
use henkei_runtime::config::{default_user_agent, AppConfig};
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum PlatformArg {
    Current,
    Linux,
    Macos,
    Windows,
}

impl From<PlatformArg> for TargetPlatform {
    fn from(arg: PlatformArg) -> Self {
        match arg {
            PlatformArg::Current => TargetPlatform::Current,
            PlatformArg::Linux => TargetPlatform::Linux,
            PlatformArg::Macos => TargetPlatform::MacOS,
            PlatformArg::Windows => TargetPlatform::Windows,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "henkei")]
#[command(
    author = "Henkei Contributors",
    version = "0.2.0",
    about = "Universal cross-platform web-to-desktop metamorphosis engine."
)]
struct Args {
    /// Target website URL to convert (e.g. https://news.ycombinator.com)
    url: Option<String>,

    /// Custom application name (auto-detected from manifest/title if omitted)
    #[arg(short, long)]
    name: Option<String>,

    /// Destination directory for generated package
    #[arg(short, long, default_value = "./dist")]
    output_dir: String,

    /// Initial window width
    #[arg(long, default_value_t = 1200)]
    width: u32,

    /// Initial window height
    #[arg(long, default_value_t = 800)]
    height: u32,

    /// Target OS platform to build for
    #[arg(short, long, value_enum, default_value_t = PlatformArg::Current)]
    target: PlatformArg,

    /// Install directly to system desktop / applications after building
    #[arg(long)]
    install: bool,

    /// Custom user-agent string
    #[arg(long)]
    user_agent: Option<String>,

    /// Custom CSS snippet or file path to inject into the webview
    #[arg(long)]
    inject_css: Option<String>,

    /// Custom JavaScript snippet or file path to inject into the webview
    #[arg(long)]
    inject_js: Option<String>,

    /// Additional domains permitted for internal navigation
    #[arg(long)]
    allow_domain: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("\n\x1b[31;1m✘ Error: {}\x1b[0m", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[36;1m==================================================");
    println!("   Henkei (変化) v0.2.0 - Universal Desktop Metamorphosis");
    println!("==================================================\x1b[0m");

    let mut url = args.url;
    let mut custom_name = args.name;
    let mut output_dir = PathBuf::from(args.output_dir);
    let mut target_platform: TargetPlatform = args.target.into();
    let mut install_desktop = args.install;
    let width = args.width;
    let height = args.height;

    // Interactive Wizard if no URL provided
    if url.is_none() {
        println!("🔗 Enter website URL (e.g. https://news.ycombinator.com):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err("URL cannot be empty!".into());
        }
        url = Some(trimmed);

        println!("🏷️ Enter custom app name (optional, press Enter to auto-detect):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            custom_name = Some(trimmed);
        }

        println!("🎯 Target platform [1: Current OS (Default), 2: Linux, 3: macOS, 4: Windows]:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        match input.trim() {
            "2" => target_platform = TargetPlatform::Linux,
            "3" => target_platform = TargetPlatform::MacOS,
            "4" => target_platform = TargetPlatform::Windows,
            _ => target_platform = TargetPlatform::Current,
        }

        println!("📁 Output directory (optional, press Enter for './dist'):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            output_dir = PathBuf::from(trimmed);
        }

        #[cfg(target_os = "linux")]
        {
            println!("🚀 Install desktop shortcut to ~/.local/share/applications? [y/N]:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().eq_ignore_ascii_case("y") {
                install_desktop = true;
            }
        }
    }

    let target_url = url.unwrap();
    println!("\n\x1b[33m[1/3] Scraping metadata and assets from {}...\x1b[0m", target_url);
    let scraped = henkei_scraper::scrape_web_app(&target_url)?;

    let app_name = custom_name.unwrap_or(scraped.name);
    println!("  ✔ App Name: \x1b[32;1m{}\x1b[0m", app_name);
    if let Some(desc) = &scraped.description {
        println!("  ✔ Description: {}", desc);
    }
    if let Some(color) = &scraped.theme_color {
        println!("  ✔ Theme Color: {}", color);
    }
    if let Some(icon) = &scraped.icon {
        println!("  ✔ High-Res Icon: {}x{} RGBA", icon.width(), icon.height());
    } else {
        println!("  ⚠ No icon detected, using default OS icon");
    }

    // 2. Prepare CSS/JS injection
    let inject_css = args.inject_css.map(|val| {
        if Path::new(&val).exists() {
            std::fs::read_to_string(&val).unwrap_or(val)
        } else {
            val
        }
    });

    let inject_js = args.inject_js.map(|val| {
        if Path::new(&val).exists() {
            std::fs::read_to_string(&val).unwrap_or(val)
        } else {
            val
        }
    });

    // 3. Construct AppConfig
    let user_agent = args.user_agent.unwrap_or_else(default_user_agent);
    let config = AppConfig {
        name: app_name.clone(),
        url: scraped.url,
        width,
        height,
        user_agent,
        allowed_domains: args.allow_domain,
        devtools: true,
        persist_geometry: true,
        inject_css,
        inject_js,
    };

    // 4. Locate or build runtime stub
    println!("\n\x1b[33m[2/3] Preparing runtime runner stub...\x1b[0m");
    let runtime_bin = get_or_build_runtime()?;
    println!("  ✔ Runtime Runner: {}", runtime_bin.display());

    // 5. Build native application package
    println!("\n\x1b[33m[3/3] Packaging native desktop deliverable...\x1b[0m");
    std::fs::create_dir_all(&output_dir)?;
    let output_dir = std::fs::canonicalize(&output_dir).unwrap_or(output_dir);
    let bundle_path = henkei_builder::build_app(
        target_platform,
        &config,
        scraped.icon.as_ref(),
        &runtime_bin,
        &output_dir,
    )?;

    println!("  ✔ Generated Deliverable: \x1b[32;1m{}\x1b[0m", bundle_path.display());

    // 6. Optional desktop integration
    if install_desktop {
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = henkei_builder::linux::install_to_freedesktop(&config, &bundle_path) {
                eprintln!("  ⚠ Desktop installation warning: {}", e);
            } else {
                println!("  ✔ Installed to system applications menu!");
            }
        }
    }

    println!("\n\x1b[32;1m🎉 Metamorphosis Complete!\x1b[0m");
    println!("Your desktop app is ready to launch at: {}\n", bundle_path.display());

    Ok(())
}

fn get_or_build_runtime() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 1. Check beside current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join(format!("henkei-runtime{}", std::env::consts::EXE_SUFFIX));
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // 2. Check workspace target directory
    let target_release = PathBuf::from("target/release").join(format!("henkei-runtime{}", std::env::consts::EXE_SUFFIX));
    if target_release.exists() {
        return Ok(target_release);
    }

    let target_debug = PathBuf::from("target/debug").join(format!("henkei-runtime{}", std::env::consts::EXE_SUFFIX));
    if target_debug.exists() {
        return Ok(target_debug);
    }

    // 3. Check user cache (~/.henkei/cache/runners/henkei-runtime)
    if let Some(home) = dirs::home_dir() {
        let cached = home
            .join(".henkei")
            .join("cache")
            .join("runners")
            .join(format!("henkei-runtime{}", std::env::consts::EXE_SUFFIX));
        if cached.exists() {
            return Ok(cached);
        }
    }

    // 4. Build via Cargo
    println!("  Building henkei-runtime runner via Cargo...");
    let status = std::process::Command::new("cargo")
        .args(["build", "--release", "-p", "henkei-runtime"])
        .status()?;

    if status.success() && target_release.exists() {
        if let Some(home) = dirs::home_dir() {
            let cache_dir = home.join(".henkei").join("cache").join("runners");
            let _ = std::fs::create_dir_all(&cache_dir);
            let _ = std::fs::copy(
                &target_release,
                cache_dir.join(format!("henkei-runtime{}", std::env::consts::EXE_SUFFIX)),
            );
        }
        return Ok(target_release);
    }

    Err("Failed to locate or build henkei-runtime executable".into())
}
