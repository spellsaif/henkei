use henkei_runtime::config::AppConfig;
use henkei_scraper::icon::save_as_icns;
use image::DynamicImage;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_macos_bundle(
    config: &AppConfig,
    icon: Option<&DynamicImage>,
    runtime_bin: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let sanitized = henkei_scraper::sanitize_name(&config.name);
    let app_bundle_name = format!("{}.app", config.name);
    let app_bundle_dir = output_dir.join(&app_bundle_name);

    let contents_dir = app_bundle_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");

    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;

    // 1. Copy runtime executable into MacOS/
    let target_bin = macos_dir.join(&sanitized);
    fs::copy(runtime_bin, &target_bin)?;
    let _ = crate::append_config_trailer(&target_bin, config);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_bin, perms)?;
    }

    // 2. Write config.json in Resources/
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(resources_dir.join("config.json"), config_json)?;

    // 3. Write app.icns in Resources/
    if let Some(img) = icon {
        let icns_path = resources_dir.join("app.icns");
        let _ = save_as_icns(img, &icns_path);
    }

    // 4. Generate standard Info.plist
    let bundle_id = format!("com.henkei.{}", sanitized.to_lowercase());
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{name}</string>
    <key>CFBundleDisplayName</key>
    <string>{name}</string>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleExecutable</key>
    <string>{sanitized}</string>
    <key>CFBundleIconFile</key>
    <string>app.icns</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
"#,
        name = config.name,
        bundle_id = bundle_id,
        sanitized = sanitized
    );

    fs::write(contents_dir.join("Info.plist"), plist_content)?;

    Ok(app_bundle_dir)
}
