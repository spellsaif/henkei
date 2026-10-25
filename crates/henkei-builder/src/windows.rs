use henkei_runtime::config::AppConfig;
use henkei_scraper::icon::save_as_ico;
use image::DynamicImage;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_windows_bundle(
    config: &AppConfig,
    icon: Option<&DynamicImage>,
    runtime_bin: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let sanitized = henkei_scraper::sanitize_name(&config.name);
    let bundle_dir = output_dir.join(&sanitized);
    fs::create_dir_all(&bundle_dir)?;

    // 1. Copy runtime executable
    let target_bin = bundle_dir.join(format!("{}.exe", sanitized));
    fs::copy(runtime_bin, &target_bin)?;
    let _ = crate::append_config_trailer(&target_bin, config);

    // 2. Write config.json
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(bundle_dir.join("config.json"), config_json)?;

    // 3. Save app.ico
    if let Some(img) = icon {
        let ico_path = bundle_dir.join("app.ico");
        let _ = save_as_ico(img, &ico_path);
    }

    // 4. Generate Windows shortcut batch script for convenience
    let install_bat = format!(
        "@echo off\r\n\
         echo Installing {name}...\r\n\
         powershell -NoProfile -ExecutionPolicy Bypass -Command \"$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut([System.IO.Path]::Combine([Environment]::GetFolderPath('Desktop'), '{name}.lnk')); $Shortcut.TargetPath = '%~dp0{sanitized}.exe'; if (Test-Path '%~dp0app.ico') {{ $Shortcut.IconLocation = '%~dp0app.ico' }}; $Shortcut.Save()\"\r\n\
         echo Shortcut created on Desktop!\r\n\
         pause\r\n",
        name = config.name,
        sanitized = sanitized
    );
    let _ = fs::write(bundle_dir.join("create-desktop-shortcut.bat"), install_bat);

    Ok(bundle_dir)
}

#[cfg(target_os = "windows")]
pub fn create_windows_shortcut(
    target_exe: &Path,
    shortcut_path: &Path,
    icon_path: Option<&Path>,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows_sys::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);

        let mut shell_link: *mut IShellLinkW = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &ShellLink,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IShellLinkW::IID,
            &mut shell_link as *mut _ as *mut _,
        );

        if hr >= 0 && !shell_link.is_null() {
            let target_w: Vec<u16> = target_exe.as_os_str().encode_wide().chain(Some(0)).collect();
            ((*(*shell_link).lpVtbl).SetPath)(shell_link, target_w.as_ptr());

            let desc_w: Vec<u16> = std::ffi::OsStr::new(description)
                .encode_wide()
                .chain(Some(0))
                .collect();
            ((*(*shell_link).lpVtbl).SetDescription)(shell_link, desc_w.as_ptr());

            if let Some(ico) = icon_path {
                let ico_w: Vec<u16> = ico.as_os_str().encode_wide().chain(Some(0)).collect();
                ((*(*shell_link).lpVtbl).SetIconLocation)(shell_link, ico_w.as_ptr(), 0);
            }

            // Query IPersistFile
            let mut persist_file: *mut windows_sys::Win32::Storage::FileSystem::IPersistFile =
                std::ptr::null_mut();
            let hr_pf = ((*(*shell_link).lpVtbl).QueryInterface)(
                shell_link as *mut _,
                &windows_sys::Win32::Storage::FileSystem::IPersistFile::IID,
                &mut persist_file as *mut _ as *mut _,
            );

            if hr_pf >= 0 && !persist_file.is_null() {
                let save_w: Vec<u16> = shortcut_path
                    .as_os_str()
                    .encode_wide()
                    .chain(Some(0))
                    .collect();
                ((*(*persist_file).lpVtbl).Save)(persist_file, save_w.as_ptr(), 1);
                ((*(*persist_file).lpVtbl).Release)(persist_file);
            }

            ((*(*shell_link).lpVtbl).Release)(shell_link);
        }

        CoUninitialize();
    }

    Ok(())
}
