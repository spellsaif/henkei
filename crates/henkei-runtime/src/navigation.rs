use url::Url;

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
        let _ = open::that_detached(target_url);
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

    // 3. User configured allowed domains
    for domain in allowed_domains {
        let d = domain.to_lowercase();
        if target_host == d || target_host.ends_with(&format!(".{}", d)) {
            return true;
        }
    }

    // 4. Common SSO/OAuth authenticators to permit seamless authentication flows
    let oauth_domains = [
        "accounts.google.com",
        "github.com",
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

    false
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
}
