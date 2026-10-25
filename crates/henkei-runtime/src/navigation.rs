use url::Url;

/// Checks if a URL should be handled internally inside the application webview.
pub fn should_navigate_internally(target_url: &str, app_root_url: &str, allowed_domains: &[String]) -> bool {
    let target = match Url::parse(target_url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    
    // Always permit local pseudo-schemes and about/data/blob
    if matches!(target.scheme(), "blob" | "data" | "about" | "javascript") {
        return true;
    }

    // Never handle non-web protocols internally (mailto:, tel:, etc.)
    if target.scheme() != "http" && target.scheme() != "https" {
        return false;
    }

    let target_host = match target.host_str() {
        Some(h) => h.to_lowercase(),
        None => return false,
    };

    let app_url = match Url::parse(app_root_url) {
        Ok(u) => u,
        Err(_) => return true,
    };

    let app_host = match app_url.host_str() {
        Some(h) => h.to_lowercase(),
        None => return true,
    };

    // 1. Exact host match
    if target_host == app_host {
        return true;
    }

    // 2. Subdomain of application host (e.g. app.slack.com -> files.slack.com)
    if target_host.ends_with(&format!(".{}", app_host)) {
        return true;
    }

    // 3. App-specific companion media CDNs (e.g. YouTube streaming endpoints)
    if app_host == "youtube.com" || app_host.ends_with(".youtube.com") || app_host == "youtu.be" {
        let yt_domains = ["youtube.com", "youtu.be", "googlevideo.com", "ytimg.com", "gstatic.com"];
        for d in &yt_domains {
            if target_host == *d || target_host.ends_with(&format!(".{}", d)) {
                return true;
            }
        }
    }

    // 4. User configured allowed domains
    for domain in allowed_domains {
        let d = domain.to_lowercase();
        if target_host == d || target_host.ends_with(&format!(".{}", d)) {
            return true;
        }
    }

    // 5. Common SSO/OAuth authenticators to permit seamless authentication flows
    let oauth_domains = [
        "accounts.google.com",
        "login.microsoftonline.com",
        "appleid.apple.com",
        "auth0.com",
        "clerk.dev",
        "clerk.com",
        "okta.com",
    ];

    for auth in &oauth_domains {
        if target_host == *auth || target_host.ends_with(&format!(".{}", auth)) {
            return true;
        }
    }

    // GitHub OAuth login flow
    if target_host == "github.com" {
        let path = target.path();
        if path.starts_with("/login") || path.starts_with("/oauth") {
            return true;
        }
    }

    false
}

/// Determines if an external link should be dispatched to the system browser.
/// Returns false for internal app links, OAuth flows, and local pseudo-schemes.
pub fn should_open_in_browser(target_url: &str, app_root_url: &str, allowed_domains: &[String]) -> bool {
    // If it navigates internally, don't open in external browser
    if should_navigate_internally(target_url, app_root_url, allowed_domains) {
        return false;
    }

    let target = match Url::parse(target_url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    if matches!(target.scheme(), "blob" | "data" | "about" | "javascript") {
        return false;
    }

    target.scheme() == "http" || target.scheme() == "https"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_navigation() {
        let app_url = "https://example.com/app";
        let allowed = vec!["cdn.example.com".to_string()];

        // Same host
        assert!(should_navigate_internally("https://example.com/dashboard", app_url, &allowed));
        // Subdomain
        assert!(should_navigate_internally("https://sub.example.com/login", app_url, &allowed));
        // Explicit allowed domain
        assert!(should_navigate_internally("https://cdn.example.com/asset.js", app_url, &allowed));
        // OAuth domain
        assert!(should_navigate_internally("https://accounts.google.com/o/oauth2", app_url, &allowed));
        assert!(should_navigate_internally("https://github.com/login", app_url, &allowed));
        // Pseudo schemes
        assert!(should_navigate_internally("about:blank", app_url, &allowed));
        assert!(should_navigate_internally("data:text/html,test", app_url, &allowed));

        // External domain
        assert!(!should_navigate_internally("https://twitter.com/user", app_url, &allowed));
        assert!(!should_navigate_internally("https://phishing-example.com", app_url, &allowed));
    }

    #[test]
    fn test_external_browser_dispatch() {
        let app_url = "https://news.ycombinator.com";
        let allowed = vec![];

        // Legitimate external links clicked by user SHOULD be dispatched to default browser
        assert!(should_open_in_browser("https://nytimes.com/article", app_url, &allowed));
        assert!(should_open_in_browser("https://github.com/spellsaif/henkei", app_url, &allowed));

        // Internal app links should NOT open in external browser
        assert!(!should_open_in_browser("https://news.ycombinator.com/item?id=123", app_url, &allowed));
        // Pseudo schemes should NOT open in browser
        assert!(!should_open_in_browser("about:blank", app_url, &allowed));
        assert!(!should_open_in_browser("javascript:void(0)", app_url, &allowed));
    }
}
