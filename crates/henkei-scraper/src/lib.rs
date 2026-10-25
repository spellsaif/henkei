pub mod html;
pub mod icon;
pub mod manifest;

use image::DynamicImage;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ScrapedApp {
    pub name: String,
    pub sanitized_name: String,
    pub url: String,
    pub description: Option<String>,
    pub theme_color: Option<String>,
    pub icon: Option<DynamicImage>,
}

pub fn sanitize_name(name: &str) -> String {
    let mut sanitized = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            sanitized.push(c);
        } else if c.is_whitespace() && !sanitized.ends_with('-') && !sanitized.is_empty() {
            sanitized.push('-');
        }
    }
    let trimmed = sanitized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "App".to_string()
    } else {
        trimmed
    }
}

pub fn scrape_web_app(target_url: &str) -> Result<ScrapedApp, Box<dyn std::error::Error>> {
    let raw_url = if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        format!("https://{}", target_url)
    } else {
        target_url.to_string()
    };

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(15))
        .build()?;

    let response = client.get(&raw_url).send()?;
    let final_url = response.url().clone();
    let body = response.text()?;

    let html_meta = html::parse_html(&body, &final_url);

    let mut manifest_data = None;
    if let Some(manifest_url) = &html_meta.manifest_url {
        if let Ok(m_res) = client.get(manifest_url.as_str()).send() {
            if m_res.status().is_success() {
                if let Ok(m_text) = m_res.text() {
                    if let Ok(manifest) = manifest::parse_manifest(&m_text) {
                        manifest_data = Some(manifest);
                    }
                }
            }
        }
    }

    // Determine Title
    let mut resolved_name = None;
    if let Some(m) = &manifest_data {
        if let Some(n) = &m.name {
            if !n.trim().is_empty() {
                resolved_name = Some(n.trim().to_string());
            }
        }
        if resolved_name.is_none() {
            if let Some(sn) = &m.short_name {
                if !sn.trim().is_empty() {
                    resolved_name = Some(sn.trim().to_string());
                }
            }
        }
    }

    if resolved_name.is_none() {
        if let Some(t) = &html_meta.title {
            if !t.trim().is_empty() {
                resolved_name = Some(t.trim().to_string());
            }
        }
    }

    if resolved_name.is_none() {
        if let Some(an) = &html_meta.app_name {
            if !an.trim().is_empty() {
                resolved_name = Some(an.trim().to_string());
            }
        }
    }

    let final_name = resolved_name.unwrap_or_else(|| {
        final_url
            .host_str()
            .unwrap_or("App")
            .split('.')
            .next()
            .unwrap_or("App")
            .to_string()
    });

    let sanitized = sanitize_name(&final_name);

    // Theme color & description
    let theme_color = manifest_data
        .as_ref()
        .and_then(|m| m.theme_color.clone())
        .or(html_meta.theme_color);

    let description = manifest_data
        .as_ref()
        .and_then(|m| m.description.clone())
        .or(html_meta.description);

    // Collect Icon Candidates
    let mut candidates: Vec<icon::ScrapedIconCandidate> = Vec::new();

    // From Manifest
    if let Some(m) = &manifest_data {
        for icon in &m.icons {
            if let Ok(resolved) = final_url.join(&icon.src) {
                candidates.push(icon::ScrapedIconCandidate {
                    url: resolved,
                    size_score: icon.max_size(),
                    rel: "manifest".to_string(),
                });
            }
        }
    }

    // From HTML
    for link in html_meta.icon_links {
        let score = icon::parse_sizes_to_score(link.sizes.as_deref(), &link.rel);
        candidates.push(icon::ScrapedIconCandidate {
            url: link.url,
            size_score: score,
            rel: link.rel,
        });
    }

    // Fallback: /favicon.ico
    if let Ok(fav) = final_url.join("/favicon.ico") {
        if !candidates.iter().any(|c| c.url == fav) {
            candidates.push(icon::ScrapedIconCandidate {
                url: fav,
                size_score: 32,
                rel: "favicon".to_string(),
            });
        }
    }

    // Sort descending by score
    candidates.sort_by(|a, b| b.size_score.cmp(&a.size_score));

    // Try downloading the best icon
    let mut decoded_icon = None;
    for cand in candidates {
        if let Ok(img) = icon::download_and_decode_icon(&client, &cand.url) {
            decoded_icon = Some(img);
            break;
        }
    }

    Ok(ScrapedApp {
        name: final_name,
        sanitized_name: sanitized,
        url: final_url.to_string(),
        description,
        theme_color,
        icon: decoded_icon,
    })
}
