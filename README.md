# Henkei (変化) v2
### *The Art of Universal Web-to-Desktop Metamorphosis*

**Henkei** (Japanese for *transformation* or *metamorphosis*) is an ultra-fast, cross-platform engine written in Rust that transforms any website or PWA into a lightweight, native desktop application for **Linux, macOS, and Windows**.

---

## 🎨 The Henkei Philosophy

Modern desktop apps have an Electron bloat problem. Packaging a web app often means dragging along a dedicated 150MB+ Chromium browser instance and Node.js runtime for every single app. Running Slack, Discord, and Notion simultaneously means your machine runs three redundant web browsers, burning gigabytes of RAM and battery life.

**Henkei rejects this bloat.**

| Dimension | Electron / Nativefier | Pake | Henkei v1 | **Henkei v2** |
| :--- | :--- | :--- | :--- | :--- |
| **Runtime Engine** | Bundled Chromium + Node | OS Webview (Tauri) | Windows wry 0.24 | **Universal OS Webview (`wry 0.57+`)** |
| **Binary Footprint** | ~120 MB – 250 MB | ~15 MB | ~1.5 MB | **~1.2 MB – 3.5 MB** |
| **RAM Footprint** | 150 MB – 450 MB | 30 MB – 80 MB | ~30 MB | **~20 MB – 45 MB** |
| **Target OS Support** | Win, macOS, Linux | Win, macOS, Linux | Windows only | **Linux, macOS, Windows** |
| **Build Latency** | 3 – 5 seconds | 2 – 5 minutes | 1 – 4 minutes | **< 1 second (Cached Runner Stub)** |
| **OAuth / Google SSO** | Works | Fragile | Broken | **Hardened (Isolated Popup & Engine Bypass)** |
| **External Link Routing** | Configurable | Manual scripts | None | **Automated Domain Sandbox** |
| **Window State Memory** | Supported | Limited | None | **Native Persistent Geometry Cache** |
| **Linux Engine** | Bundled Chromium | Deprecated soup2 | Broken soup2 | **Modern WebKitGTK 4.1 + libsoup-3** |

---

## 🚀 Key Features in Henkei v2

### 1. Universal Cross-Platform Metamorphosis
- **Linux**: Produces native GTK3 / WebKitGTK 4.1 desktop packages. Integrates with FreeDesktop (`~/.local/share/applications/*.desktop`) and high-resolution icons (`hicolor` hierarchy).
- **macOS**: Generates standard Apple bundles (`AppName.app`) with custom `Info.plist`, native menus, and multi-scale Apple `.icns` icons.
- **Windows**: Produces standalone Windows executables with multi-layered `.ico` icons and native Win32 COM shell links without PowerShell script heuristics.

### 2. Smart Navigation Guard & Domain Sandbox
In a browser, clicking a link navigates to a new page. In a desktop app, clicking an external link is a context switch.
- **Internal Domains**: Subdomains and configured allowed domains render fluidly inside the webview.
- **External Links**: Third-party links automatically open in the user's default OS browser.
- **OAuth & Google SSO**: Built-in support for Google, GitHub, Microsoft, Apple, and Clerk OAuth authentication flows without `disallowed_useragent` blocks.

### 3. PWA Manifest & High-Res Asset Discovery
Henkei crawls:
- Web App Manifests (`manifest.json`) for names, theme colors, and vector/maskable icons.
- Apple Touch Icons (`<link rel="apple-touch-icon" sizes="...">`).
- OpenGraph media (`<meta property="og:image">`).
- Automatically generates multi-resolution **`.ico`** (Windows), **`.icns`** (macOS), and **PNG sets** (Linux).

### 4. Native Desktop Polish
- **Zero-Dependency Standalone Binaries**: Every compiled deliverable embeds its configuration directly in a binary overlay trailer. You can move the single executable file anywhere and it just works—no external `config.json` needed.
- **Window Geometry Memory**: Automatically remembers and restores window position, size, and maximized state across restarts.
- **Desktop Application CLI Flags**: Generated apps support `--help`, `--version`, `--install`, `--devtools`, and `--url <URL>`.
- **Client-Side Navigation Shortcuts**: Built-in `Alt + Left` (Back), `Alt + Right` (Forward), and `Ctrl+R` / `F5` (Reload).
- **Fullscreen Video**: Intercepts HTML5 fullscreen video requests (YouTube, Twitch, Netflix) and toggles native borderless fullscreen.
- **Custom Injections**: Inject custom CSS and JavaScript stylesheets and scripts seamlessly.

---

## 📥 Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/henkei.git
cd henkei

# Compile and install CLI to your PATH
cargo install --path henkei
```

---

## 📖 How to Use Henkei

### Mode A: Interactive Guided Wizard
Launch `henkei` with no arguments to enter the interactive console wizard:
```bash
henkei
```
The wizard will:
1. Prompt for the website URL.
2. Auto-discover the application name, description, theme color, and high-res icon.
3. Prompt for custom name or target OS platform.
4. Build the application and optionally install it to your system desktop menu!

### Mode B: Direct Command Line (Automation & Scripting)

#### 1. Basic Conversion
```bash
henkei https://news.ycombinator.com
```

#### 2. Specify Custom Name and Output Directory
```bash
henkei https://github.com --name "GitHub Desktop" --output-dir ./dist
```

#### 3. Cross-Target Packaging
Target another OS package structure:
```bash
# Build a macOS .app bundle
henkei https://notion.so --name "Notion" --target macos --output-dir ./dist

# Build a Windows package
henkei https://slack.com --name "Slack" --target windows --output-dir ./dist

# Build a Linux desktop package with automatic installation
henkei https://excalidraw.com --target linux --install
```

#### 4. Custom CSS & JavaScript Injection
Inject custom styles or dark modes:
```bash
henkei https://example.com --inject-css "body { filter: invert(90%); }"
```

#### 5. Additional Allowed Navigation Domains
```bash
henkei https://mycompany.com --allow-domain "login.mycompany.com" --allow-domain "cdn.mycompany.com"
```

---

## 🏗️ Architecture

Henkei v2 is organized as a modular Rust workspace:

```
henkei/
├── henkei/                       # CLI frontend and interactive wizard
└── crates/
    ├── henkei-runtime/           # Universal webview runner (wry 0.57 + tao 0.37)
    ├── henkei-scraper/           # PWA manifest, OpenGraph & multi-format icon engine
    └── henkei-builder/           # Packaging & desktop installer engine (Linux, macOS, Windows)
```

---

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
