use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub url: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default = "default_true")]
    pub devtools: bool,
    #[serde(default = "default_true")]
    pub persist_geometry: bool,
    #[serde(default)]
    pub inject_css: Option<String>,
    #[serde(default)]
    pub inject_js: Option<String>,
}

fn default_width() -> u32 {
    1200
}

fn default_height() -> u32 {
    800
}

fn default_true() -> bool {
    true
}

pub fn default_user_agent() -> String {
    #[cfg(target_os = "windows")]
    return "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string();
    
    #[cfg(target_os = "macos")]
    return "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15".to_string();
    
    #[cfg(target_os = "linux")]
    return "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string();
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Henkei App".to_string(),
            url: "https://news.ycombinator.com".to_string(),
            width: default_width(),
            height: default_height(),
            user_agent: default_user_agent(),
            allowed_domains: Vec::new(),
            devtools: true,
            persist_geometry: true,
            inject_css: None,
            inject_js: None,
        }
    }
}

pub const HENKEI_MAGIC: &[u8; 8] = b"HENKEI\0\x01";

impl AppConfig {
    /// Reads configuration from an open reader seeking the binary trailer.
    pub fn read_from_reader<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Option<Self> {
        use std::io::SeekFrom;
        let file_len = reader.seek(SeekFrom::End(0)).ok()?;
        if file_len < 12 {
            return None;
        }

        // Seek to end - 8 to check magic
        reader.seek(SeekFrom::End(-8)).ok()?;
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic).ok()?;
        if &magic != HENKEI_MAGIC {
            return None;
        }

        // Seek to end - 12 to read length
        reader.seek(SeekFrom::End(-12)).ok()?;
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).ok()?;
        let payload_len = u32::from_le_bytes(len_bytes) as u64;

        if payload_len == 0 || payload_len > file_len - 12 {
            return None;
        }

        // Seek to start of payload
        let offset = -(12 + payload_len as i64);
        reader.seek(SeekFrom::End(offset)).ok()?;
        let mut payload = vec![0u8; payload_len as usize];
        reader.read_exact(&mut payload).ok()?;

        serde_json::from_slice::<AppConfig>(&payload).ok()
    }

    /// Reads configuration from a binary trailer appended to the executable.
    /// Trailer format: [JSON payload bytes] [4-byte LE length] [8-byte HENKEI_MAGIC]
    pub fn read_from_binary_trailer() -> Option<Self> {
        let exe_path = std::env::current_exe().ok()?;
        let mut file = std::fs::File::open(exe_path).ok()?;
        Self::read_from_reader(&mut file)
    }

    /// Loads configuration with the following priority:
    /// 1. Adjacent config.json (highest precedence for quick local overrides)
    /// 2. macOS bundle Resources/config.json
    /// 3. Embedded binary trailer (for zero-dependency standalone binaries)
    /// 4. Embedded compile-time config (if compiled with feature)
    /// 5. Fallback defaults
    pub fn load() -> Self {
        // Check adjacent config.json
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let adjacent = parent.join("config.json");
                if adjacent.exists() {
                    if let Ok(content) = std::fs::read_to_string(&adjacent) {
                        if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                            return cfg;
                        }
                    }
                }
                
                // Check macOS Contents/Resources/config.json
                #[cfg(target_os = "macos")]
                {
                    let macos_res = parent.join("../Resources/config.json");
                    if macos_res.exists() {
                        if let Ok(content) = std::fs::read_to_string(&macos_res) {
                            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                                return cfg;
                            }
                        }
                    }
                }
            }
        }

        // Check standalone binary trailer
        if let Some(cfg) = Self::read_from_binary_trailer() {
            return cfg;
        }

        // Check if an embedded compile-time config exists
        #[cfg(feature = "embedded-config")]
        {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(include_str!("../embedded_config.json")) {
                return cfg;
            }
        }

        Self::default()
    }

    pub fn get_app_data_dir(&self) -> PathBuf {
        let sanitized: String = self.name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Henkei")
            .join("Apps")
            .join(sanitized);
        let _ = std::fs::create_dir_all(&base);
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_binary_trailer_roundtrip() {
        let mut cfg = AppConfig::default();
        cfg.name = "Test Metamorphosis".to_string();
        cfg.url = "https://example.com".to_string();

        let json = serde_json::to_vec(&cfg).unwrap();
        let len = json.len() as u32;

        let mut buf = Vec::new();
        // Fake binary bytes
        buf.extend_from_slice(b"FAKE_ELF_OR_PE_EXECUTABLE_BINARY_DATA_HERE");
        buf.extend_from_slice(&json);
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(HENKEI_MAGIC);

        let mut cursor = Cursor::new(buf);
        let parsed = AppConfig::read_from_reader(&mut cursor).expect("should parse config trailer");
        assert_eq!(parsed.name, "Test Metamorphosis");
        assert_eq!(parsed.url, "https://example.com");
    }
}
