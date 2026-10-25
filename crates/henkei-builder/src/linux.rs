use henkei_runtime::config::AppConfig;
use henkei_scraper::icon::save_as_png;
use image::DynamicImage;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_linux_bundle(
    config: &AppConfig,
    icon: Option<&DynamicImage>,
    runtime_bin: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let sanitized = henkei_scraper::sanitize_name(&config.name);
    let bundle_dir = output_dir.join(&sanitized);
    fs::create_dir_all(&bundle_dir)?;

    // 1. Copy or write runtime executable
    let target_bin = bundle_dir.join(&sanitized);
    fs::copy(runtime_bin, &target_bin)?;
    let _ = crate::append_config_trailer(&target_bin, config);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_bin, perms)?;
    }

    // 2. Write config.json
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(bundle_dir.join("config.json"), config_json)?;

    // 3. Save icons
    let icon_path = bundle_dir.join("icon.png");
    if let Some(img) = icon {
        let _ = save_as_png(img, &icon_path, 256);
    }

    // 4. Generate .desktop file
    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec=\"{}\"\n\
         Icon={}\n\
         Terminal=false\n\
         Categories=Network;WebBrowser;\n\
         Comment=Native desktop webapp powered by Henkei\n",
        config.name,
        target_bin.display(),
        icon_path.display()
    );
    let desktop_path = bundle_dir.join(format!("{}.desktop", sanitized));
    fs::write(&desktop_path, desktop_content)?;

    // 5. Generate local user install script
    let install_sh = format!(
        "#!/usr/bin/env bash\n\
         set -e\n\
         DESKTOP_DIR=\"$HOME/.local/share/applications\"\n\
         ICON_DIR=\"$HOME/.local/share/icons/hicolor/256x256/apps\"\n\
         mkdir -p \"$DESKTOP_DIR\" \"$ICON_DIR\"\n\
         cp \"$(dirname \"$0\")/icon.png\" \"$ICON_DIR/{sanitized}.png\" 2>/dev/null || true\n\
         sed -e \"s|Icon=.*|Icon=$ICON_DIR/{sanitized}.png|\" \"$(dirname \"$0\")/{sanitized}.desktop\" > \"$DESKTOP_DIR/{sanitized}.desktop\"\n\
         chmod +x \"$DESKTOP_DIR/{sanitized}.desktop\"\n\
         echo \"Installed {name} to desktop applications!\"\n",
        sanitized = sanitized,
        name = config.name
    );
    let install_sh_path = bundle_dir.join("install.sh");
    fs::write(&install_sh_path, install_sh)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&install_sh_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_sh_path, perms)?;

        // AppRun symlink for instant AppImage / AppDir compatibility
        let app_run = bundle_dir.join("AppRun");
        let _ = fs::remove_file(&app_run);
        let _ = std::os::unix::fs::symlink(&sanitized, &app_run);
    }

    Ok(bundle_dir)
}

pub fn install_to_freedesktop(
    config: &AppConfig,
    bundle_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sanitized = henkei_scraper::sanitize_name(&config.name);
    let bin_path = bundle_dir.join(&sanitized);

    if let Some(data_dir) = dirs::data_dir() {
        let apps_dir = data_dir.join("applications");
        let icons_dir = data_dir.join("icons").join("hicolor").join("256x256").join("apps");
        fs::create_dir_all(&apps_dir)?;
        fs::create_dir_all(&icons_dir)?;

        let target_icon = icons_dir.join(format!("{}.png", sanitized));
        let src_icon = bundle_dir.join("icon.png");
        if src_icon.exists() {
            let _ = fs::copy(src_icon, &target_icon);
        }

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Exec=\"{}\"\n\
             Icon={}\n\
             Terminal=false\n\
             Categories=Network;WebBrowser;\n\
             Comment=Native desktop webapp powered by Henkei\n",
            config.name,
            bin_path.display(),
            target_icon.display()
        );

        let desktop_file = apps_dir.join(format!("{}.desktop", sanitized));
        fs::write(desktop_file, desktop_content)?;
    }

    Ok(())
}
