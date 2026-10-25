use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebAppManifest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub start_url: Option<String>,
    pub theme_color: Option<String>,
    pub background_color: Option<String>,
    #[serde(default)]
    pub icons: Vec<ManifestIcon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestIcon {
    pub src: String,
    pub sizes: Option<String>,
    #[serde(rename = "type")]
    pub mime_type: Option<String>,
    pub purpose: Option<String>,
}

impl ManifestIcon {
    /// Estimates maximum dimension from sizes string (e.g. "512x512", "192x192 512x512", "any")
    pub fn max_size(&self) -> u32 {
        let sizes = match &self.sizes {
            Some(s) => s.trim(),
            None => return 48,
        };

        if sizes.eq_ignore_ascii_case("any") {
            return 1024;
        }

        let mut max = 48;
        for part in sizes.split_whitespace() {
            if let Some((w, _)) = part.split_once('x') {
                if let Ok(val) = w.parse::<u32>() {
                    if val > max {
                        max = val;
                    }
                }
            }
        }
        max
    }
}

pub fn parse_manifest(json_content: &str) -> Result<WebAppManifest, serde_json::Error> {
    serde_json::from_str(json_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let json = r##"{
            "name": "Super Note PWA",
            "short_name": "Note",
            "theme_color": "#123456",
            "icons": [
                { "src": "/icon-192.png", "sizes": "192x192", "type": "image/png" },
                { "src": "/icon-512.png", "sizes": "512x512", "type": "image/png" },
                { "src": "/icon-maskable.svg", "sizes": "any", "type": "image/svg+xml" }
            ]
        }"##;

        let manifest = parse_manifest(json).expect("valid manifest");
        assert_eq!(manifest.name.as_deref(), Some("Super Note PWA"));
        assert_eq!(manifest.short_name.as_deref(), Some("Note"));
        assert_eq!(manifest.theme_color.as_deref(), Some("#123456"));
        assert_eq!(manifest.icons.len(), 3);
        assert_eq!(manifest.icons[0].max_size(), 192);
        assert_eq!(manifest.icons[1].max_size(), 512);
        assert_eq!(manifest.icons[2].max_size(), 1024);
    }
}
