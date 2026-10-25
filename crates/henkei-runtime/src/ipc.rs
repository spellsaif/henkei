pub fn build_initialization_script(inject_css: Option<&str>, inject_js: Option<&str>) -> String {
    let mut script = String::from(r#"
        // Henkei Desktop Integration Engine
        (function() {
            // 1. Desktop Keyboard Shortcuts
            window.addEventListener('keydown', (e) => {
                // Navigation
                if (e.altKey && e.key === 'ArrowLeft') {
                    window.history.back();
                }
                if (e.altKey && e.key === 'ArrowRight') {
                    window.history.forward();
                }
                // Reload: Ctrl+R or F5
                if ((e.ctrlKey || e.metaKey) && (e.key === 'r' || e.key === 'R') || e.key === 'F5') {
                    e.preventDefault();
                    window.location.reload();
                }
            });

            // 2. Fullscreen Video Interceptor (YouTube, Twitch, Netflix)
            function handleFullscreen() {
                const isFull = !!(document.fullscreenElement || document.webkitFullscreenElement);
                if (window.ipc) {
                    window.ipc.postMessage(isFull ? 'henkei:fullscreen:true' : 'henkei:fullscreen:false');
                }
            }
            document.addEventListener('fullscreenchange', handleFullscreen);
            document.addEventListener('webkitfullscreenchange', handleFullscreen);

            // 3. Dynamic Title Observer
            function notifyTitle() {
                if (window.ipc && document.title) {
                    window.ipc.postMessage('henkei:title:' + document.title);
                }
            }
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', notifyTitle);
            } else {
                notifyTitle();
            }

            const titleEl = document.querySelector('title');
            if (titleEl) {
                const observer = new MutationObserver(notifyTitle);
                observer.observe(titleEl, { childList: true });
            }

            // 4. Intentional User Click Interceptor
            // Only user-initiated clicks on <a> hyperlinks will ever be considered for external browser dispatch.
            // Invisible iframes, ad scripts, and tracking beacons will NEVER hijack the host browser.
            document.addEventListener('click', (event) => {
                const anchor = event.target.closest('a');
                if (!anchor) return;

                const href = anchor.getAttribute('href');
                if (!href || href.startsWith('#') || href.startsWith('javascript:')) return;

                try {
                    const targetUrl = new URL(anchor.href, window.location.href);
                    if (targetUrl.protocol === 'http:' || targetUrl.protocol === 'https:') {
                        if (targetUrl.origin !== window.location.origin) {
                            if (window.ipc) {
                                event.preventDefault();
                                event.stopPropagation();
                                window.ipc.postMessage('henkei:open_external:' + targetUrl.href);
                            }
                        }
                    }
                } catch (_) {}
            }, true);

            // 5. Automated Video Player Ad-Skipper (Pake-Style Automation)
            // Fast-forwards in-stream ads without breaking the media pipeline
            (function initVideoPlayerAdSkipper() {
                const isYouTube = window.location.hostname.includes('youtube.com');
                if (!isYouTube) return;

                // Inject cosmetic banner hiding CSS
                const adStyle = document.createElement('style');
                adStyle.id = 'henkei-cosmetic-shield';
                adStyle.textContent = `
                    #masthead-ad, 
                    ytd-ad-slot-renderer, 
                    .ytd-in-feed-ad-layout-renderer,
                    .ytp-ad-overlay-container,
                    .ytp-ad-message-container,
                    ytd-banner-promo-renderer,
                    #player-ads {
                        display: none !important;
                    }
                `;
                (document.head || document.documentElement).appendChild(adStyle);

                // Auto-skipper interval
                setInterval(() => {
                    const video = document.querySelector('video');
                    const adShowing = document.querySelector('.ad-showing, .ad-interrupting, .ytp-ad-player-overlay');
                    if (adShowing && video) {
                        video.muted = true;
                        video.playbackRate = 16.0;
                        if (!isNaN(video.duration) && isFinite(video.duration)) {
                            video.currentTime = video.duration;
                        }
                    }

                    // Click skip button immediately when available
                    const skipButton = document.querySelector(
                        '.ytp-ad-skip-button, .ytp-ad-skip-button-modern, .ytp-skip-ad-button'
                    );
                    if (skipButton) {
                        skipButton.click();
                    }
                }, 100);
            })();
        })();
    "#);

    // 4. Custom CSS Injection
    if let Some(css) = inject_css {
        let escaped_css = css.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        script.push_str(&format!(
            r#"
            (function() {{
                const style = document.createElement('style');
                style.id = 'henkei-custom-styles';
                style.textContent = `{}`;
                if (document.head) {{
                    document.head.appendChild(style);
                }} else {{
                    document.addEventListener('DOMContentLoaded', () => document.head.appendChild(style));
                }}
            }})();
            "#,
            escaped_css
        ));
    }

    // 5. Custom JS Injection
    if let Some(js) = inject_js {
        script.push_str(&format!(
            r#"
            (function() {{
                try {{
                    {}
                }} catch (e) {{
                    console.error('Henkei custom script error:', e);
                }}
            }})();
            "#,
            js
        ));
    }

    script
}
