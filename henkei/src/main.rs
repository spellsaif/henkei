use clap::Parser;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;
use std::process::Command;

mod templates;

#[derive(Parser, Debug)]
#[command(name = "henkei")]
#[command(about = "Converts any website into a native, optimized Windows desktop application with an embedded installer.", long_about = None)]
struct Args {
    /// The URL of the website to convert (e.g. https://news.ycombinator.com)
    url: Option<String>,

    /// Custom name for the application (scraped from HTML title if not specified)
    #[arg(short, long)]
    name: Option<String>,

    /// The output directory for the compiled installer
    #[arg(short, long, default_value = ".")]
    output_dir: String,

    /// Initial window width
    #[arg(long, default_value_t = 1200)]
    width: u32,

    /// Initial window height
    #[arg(long, default_value_t = 800)]
    height: u32,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("\x1b[31;1mError: {}\x1b[0m", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[36;1m==================================================");
    println!("   Henkei Web-to-Desktop App Compiler");
    println!("==================================================\x1b[0m");

    let mut url = args.url.clone();
    let mut name = args.name.clone();
    let mut width = args.width;
    let mut height = args.height;
    let mut output_dir = args.output_dir.clone();

    if url.is_none() {
        println!("🔗 Enter website URL (e.g. https://news.ycombinator.com):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err("URL cannot be empty!".into());
        }
        url = Some(trimmed);

        println!("🏷️ Enter app name (optional, press Enter to auto-detect):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            name = Some(trimmed);
        }

        println!("↕️ Enter window width (optional, press Enter for 1200):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            if let Ok(w) = trimmed.parse::<u32>() {
                width = w;
            }
        }

        println!("↔️ Enter window height (optional, press Enter for 800):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            if let Ok(h) = trimmed.parse::<u32>() {
                height = h;
            }
        }

        println!("📁 Enter output directory (optional, press Enter for './build'):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            output_dir = trimmed;
        } else {
            output_dir = "./build".to_string();
        }
    }

    let url_str = url.unwrap();
    let url_str = if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
        format!("https://{}", url_str)
    } else {
        url_str
    };

    println!("Target URL: {}", url_str);

    // 1. Fetch page metadata
    println!("\n\x1b[33m[1/5] Fetching website metadata...\x1b[0m");
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;
    
    let response = client.get(&url_str).send()?;
    let final_url = response.url().clone();
    let body = response.text()?;
    
    let document = Html::parse_document(&body);
    
    // Scrape title
    let title = if let Some(custom_name) = &name {
        custom_name.clone()
    } else {
        let title_selector = Selector::parse("title").unwrap();
        let og_title_selector = Selector::parse("meta[property='og:title']").unwrap();
        
        let mut scraped_title = document.select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
            
        if scraped_title.is_empty() {
            scraped_title = document.select(&og_title_selector)
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|s| s.to_string())
                .unwrap_or_default();
        }
        
        if scraped_title.is_empty() {
            final_url.host_str().unwrap_or("App").to_string()
        } else {
            scraped_title
        }
    };
    
    let sanitized_name = sanitize_name(&title);
    println!("App Name: {} (sanitized: {})", title, sanitized_name);

    // Scrape icons
    println!("\n\x1b[33m[2/5] Detecting website icons...\x1b[0m");
    let link_selector = Selector::parse("link").unwrap();
    let mut icons = Vec::new();
    
    for link in document.select(&link_selector) {
        if let Some(rel) = link.value().attr("rel") {
            let rel_lower = rel.to_lowercase();
            if rel_lower.contains("icon") || rel_lower == "shortcut" || rel_lower == "apple-touch-icon" {
                if let Some(href) = link.value().attr("href") {
                    let sizes = link.value().attr("sizes").unwrap_or("").to_string();
                    icons.push((href.to_string(), sizes));
                }
            }
        }
    }
    
    let mut resolved_icons = Vec::new();
    for (href, sizes) in icons {
        if let Ok(resolved) = final_url.join(&href) {
            resolved_icons.push((resolved, parse_sizes(&sizes)));
        }
    }
    
    // Always append default favicon.ico as a fallback if not already present
    if let Ok(default_fav) = final_url.join("/favicon.ico") {
        if !resolved_icons.iter().any(|(u, _)| u == &default_fav) {
            resolved_icons.push((default_fav, 32));
        }
    }
    
    // Sort by size descending (largest first)
    resolved_icons.sort_by(|a, b| b.1.cmp(&a.1));
    
    let mut icon_img: Option<image::DynamicImage> = None;
    for (icon_url, size) in &resolved_icons {
        println!("Trying icon: {} (size: {}x{})", icon_url, size, size);
        if let Ok(res) = client.get(icon_url.as_str()).send() {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes() {
                    match image::load_from_memory(&bytes) {
                        Ok(img) => {
                            icon_img = Some(img);
                            println!("Successfully downloaded and decoded icon!");
                            break;
                        }
                        Err(e) => {
                            println!("Downloaded icon but failed to parse: {}. Trying next...", e);
                        }
                    }
                }
            }
        }
    }

    // 2. Generate the .ico file
    let temp_dir = std::env::temp_dir().join(format!("henkei-build-{}", sanitized_name));
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&temp_dir)?;
    
    let temp_app_dir = temp_dir.join("app");
    let temp_inst_dir = temp_dir.join("installer");
    fs::create_dir_all(&temp_app_dir)?;
    fs::create_dir_all(&temp_inst_dir)?;

    if let Some(img) = icon_img {
        println!("\n\x1b[33m[3/5] Converting downloaded icon to Windows .ico...\x1b[0m");
        let rgba = img.to_rgba8();
        let resized = image::imageops::resize(&rgba, 256, 256, image::imageops::FilterType::Lanczos3);
        
        let app_ico_path = temp_app_dir.join("app.ico");
        let inst_ico_path = temp_inst_dir.join("app.ico");
        
        if let Ok(_) = resized.save_with_format(&app_ico_path, image::ImageFormat::Ico) {
            let _ = fs::copy(&app_ico_path, &inst_ico_path);
            println!("Icon converted successfully and saved to build contexts!");
        } else {
            eprintln!("Warning: Failed to save icon as .ico format.");
        }
    } else {
        println!("No parseable icons found on the site. Application will use default executable icon.");
    }

    // 3. Set up the temporary cargo build environment
    println!("\n\x1b[33m[4/5] Preparing build environments...\x1b[0m");
    
    // Write Webview app files
    fs::create_dir_all(temp_app_dir.join("src"))?;
    fs::write(temp_app_dir.join("Cargo.toml"), templates::APP_CARGO_TOML)?;
    fs::write(temp_app_dir.join("build.rs"), templates::APP_BUILD_RS)?;
    fs::write(temp_app_dir.join("src").join("main.rs"), templates::APP_MAIN_RS)?;
    
    // Write custom app config.rs
    let app_config_content = format!(
        "pub const APP_NAME: &str = {:?};\n\
         pub const APP_URL: &str = {:?};\n\
         pub const APP_WIDTH: u32 = {};\n\
         pub const APP_HEIGHT: u32 = {};\n\
         pub const USER_AGENT: &str = {:?};\n",
        sanitized_name,
        final_url.as_str(),
        width,
        height,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    );
    fs::write(temp_app_dir.join("src").join("config.rs"), app_config_content)?;

    // Write Installer files
    fs::create_dir_all(temp_inst_dir.join("src"))?;
    fs::write(temp_inst_dir.join("Cargo.toml"), templates::INSTALLER_CARGO_TOML)?;
    fs::write(temp_inst_dir.join("build.rs"), templates::INSTALLER_BUILD_RS)?;
    fs::write(temp_inst_dir.join("src").join("main.rs"), templates::INSTALLER_MAIN_RS)?;
    
    // Write custom installer config.rs
    let inst_config_content = format!(
        "pub const APP_NAME: &str = {:?};\n\
         pub const APP_VERSION: &str = \"1.0.0\";\n\
         pub const PUBLISHER: &str = \"Henkei Web-to-Desktop Packager\";\n",
        sanitized_name
    );
    fs::write(temp_inst_dir.join("src").join("config.rs"), inst_config_content)?;

    // 4. Compile the Webview runner binary
    println!("\n\x1b[33m[5/5] Compiling WebView executable (this may take a minute)...\x1b[0m");
    println!("Compiling WebView runner...");
    let target_cache_dir = std::env::temp_dir().join("henkei-target-cache");
    fs::create_dir_all(&target_cache_dir)?;

    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .env("CARGO_TARGET_DIR", &target_cache_dir)
        .current_dir(&temp_app_dir)
        .status()?;
        
    if !status.success() {
        return Err("Failed to compile WebView runner application".into());
    }
    
    // Copy the compiled application exe into the installer source
    let app_exe_src = target_cache_dir.join("release").join("app-template.exe");
    let app_exe_dest = temp_inst_dir.join("src").join("app_bin.exe");
    fs::copy(&app_exe_src, &app_exe_dest)?;

    // Compile the installer binary
    println!("Compiling single-file installer...");
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .env("CARGO_TARGET_DIR", &target_cache_dir)
        .current_dir(&temp_inst_dir)
        .status()?;
        
    if !status.success() {
        return Err("Failed to compile Installer application".into());
    }

    // 5. Deliver output
    let output_dir_path = Path::new(&output_dir);
    if !output_dir_path.exists() {
        fs::create_dir_all(output_dir_path)?;
    }
    
    let installer_src = target_cache_dir.join("release").join("installer-template.exe");
    let installer_name = format!("{}_installer.exe", sanitized_name);
    let installer_dest = output_dir_path.join(&installer_name);
    
    fs::copy(&installer_src, &installer_dest)?;

    // Cleanup temp source files (keep the target build cache intact)
    let _ = fs::remove_dir_all(&temp_dir);

    println!("\n\x1b[32;1mSuccess! Desktop installer generated.\x1b[0m");
    println!("Installer Binary: {}", installer_dest.display());
    println!("Share this installer with users to install your app with desktop & start-menu shortcuts.");
    
    Ok(())
}

fn sanitize_name(name: &str) -> String {
    let sanitized: String = name.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "WebDesktopApp".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_sizes(sizes: &str) -> u32 {
    if sizes.is_empty() || sizes.to_lowercase() == "any" {
        return 32;
    }
    if let Some(w_str) = sizes.split('x').next() {
        if let Ok(w) = w_str.parse::<u32>() {
            return w;
        }
    }
    32
}
