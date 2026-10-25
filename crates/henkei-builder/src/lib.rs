pub mod linux;
pub mod macos;
pub mod windows;

use henkei_runtime::config::AppConfig;
use image::DynamicImage;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    Current,
    Linux,
    MacOS,
    Windows,
}

impl Default for TargetPlatform {
    fn default() -> Self {
        TargetPlatform::Current
    }
}

pub fn append_config_trailer(bin_path: &Path, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let payload = serde_json::to_vec(config)?;
    let len = payload.len() as u32;
    let mut file = std::fs::OpenOptions::new().append(true).open(bin_path)?;
    file.write_all(&payload)?;
    file.write_all(&len.to_le_bytes())?;
    file.write_all(henkei_runtime::config::HENKEI_MAGIC)?;
    file.flush()?;
    Ok(())
}

pub fn build_app(
    platform: TargetPlatform,
    config: &AppConfig,
    icon: Option<&DynamicImage>,
    runtime_bin: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resolved_platform = match platform {
        TargetPlatform::Current => {
            #[cfg(target_os = "linux")]
            {
                TargetPlatform::Linux
            }
            #[cfg(target_os = "macos")]
            {
                TargetPlatform::MacOS
            }
            #[cfg(target_os = "windows")]
            {
                TargetPlatform::Windows
            }
            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            {
                TargetPlatform::Linux
            }
        }
        other => other,
    };

    match resolved_platform {
        TargetPlatform::Linux => linux::build_linux_bundle(config, icon, runtime_bin, output_dir),
        TargetPlatform::MacOS => macos::build_macos_bundle(config, icon, runtime_bin, output_dir),
        TargetPlatform::Windows => windows::build_windows_bundle(config, icon, runtime_bin, output_dir),
        TargetPlatform::Current => unreachable!(),
    }
}
