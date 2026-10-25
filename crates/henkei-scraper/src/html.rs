use scraper::{Html, Selector};
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct ScrapedHtmlMetadata {
    pub title: Option<String>,
    pub app_name: Option<String>,
    pub description: Option<String>,
    pub theme_color: Option<String>,
    pub manifest_url: Option<Url>,
    pub icon_links: Vec<IconLink>,
}

#[derive(Debug, Clone)]
pub struct IconLink {
    pub url: Url,
    pub rel: String,
    pub sizes: Option<String>,
    pub mime_type: Option<String>,
}

pub fn parse_html(html_str: &str, base_url: &Url) -> ScrapedHtmlMetadata {
    let document = Html::parse_document(html_str);
    let mut meta = ScrapedHtmlMetadata::default();

    // 1. Title
    if let Ok(title_sel) = Selector::parse("title") {
        if let Some(el) = document.select(&title_sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                meta.title = Some(t);
            }
        }
    }

    // 2. Meta tags
    if let Ok(meta_sel) = Selector::parse("meta") {
        for el in document.select(&meta_sel) {
            let val = el.value();
            let content = val.attr("content").map(|s| s.trim().to_string());
            let name = val.attr("name").map(|s| s.to_lowercase());
            let prop = val.attr("property").map(|s| s.to_lowercase());

            // OpenGraph Title or Application Name
            if meta.title.is_none() || meta.title.as_deref() == Some("") {
                if prop.as_deref() == Some("og:title") {
                    meta.title = content.clone();
                }
            }

            if name.as_deref() == Some("application-name") || prop.as_deref() == Some("og:site_name") {
                meta.app_name = content.clone();
            }

            if name.as_deref() == Some("theme-color") {
                meta.theme_color = content.clone();
            }

            if name.as_deref() == Some("description") || prop.as_deref() == Some("og:description") {
                if meta.description.is_none() {
                    meta.description = content.clone();
                }
            }

            // OpenGraph Image
            if prop.as_deref() == Some("og:image") {
                if let Some(img_url_str) = content {
                    if let Ok(resolved) = base_url.join(&img_url_str) {
                        meta.icon_links.push(IconLink {
                            url: resolved,
                            rel: "og:image".to_string(),
                            sizes: None,
                            mime_type: None,
                        });
                    }
                }
            }
        }
    }

    // 3. Link tags
    if let Ok(link_sel) = Selector::parse("link") {
        for el in document.select(&link_sel) {
            let val = el.value();
            let rel = match val.attr("rel") {
                Some(r) => r.to_lowercase(),
                None => continue,
            };
            let href = match val.attr("href") {
                Some(h) => h.trim(),
                None => continue,
            };

            let resolved_url = match base_url.join(href) {
                Ok(u) => u,
                Err(_) => continue,
            };

            if rel.contains("manifest") {
                meta.manifest_url = Some(resolved_url.clone());
            }

            if rel.contains("icon") || rel == "shortcut icon" || rel == "apple-touch-icon" {
                meta.icon_links.push(IconLink {
                    url: resolved_url,
                    rel,
                    sizes: val.attr("sizes").map(|s| s.to_string()),
                    mime_type: val.attr("type").map(|s| s.to_string()),
                });
            }
        }
    }

    meta
}
