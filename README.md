# Henkei (変化)
### *The Art of Web-to-Desktop Metamorphosis*

**Henkei** (Japanese for *transformation* or *metamorphosis*) is a high-performance, developer-first command-line tool written in Rust. It compiles any website or web application into a native Windows desktop application and packages it inside a clean, single-file installer.

---

## 🎨 The Henkei Philosophy

Modern desktop apps have a bloat problem. In the era of Electron, packaging a website as a desktop application means bundling a dedicated 100MB+ Chromium browser instance and a Node.js runtime for every single app. If a user runs three Electron apps (like Slack, Discord, and VS Code), their machine is running three separate web browsers, consuming gigabytes of memory and storage.

**Henkei rejects this bloat.** Our engineering philosophy is built on three pillars:

### 1. Zero Bloat, Infinite Performance
Henkei does not bundle Chromium. Instead, it dynamically links to the Windows-native **WebView2 runtime** (the Chromium-based Edge engine) that is already built into modern Windows operating systems. 
- **Binary Size**: ~1 MB for the installer, ~900 KB for the installed application.
- **RAM Footprint**: Under 30 MB of RAM (compared to 150MB+ for Electron).
- **Startup Time**: Instantaneous launching and rendering.

### 2. Zero Friction & Automated Metamorphosis
You shouldn't have to configure icons, write manifests, or edit configurations. You provide a URL; Henkei handles the rest. It fetches HTML metadata, scrapes high-resolution favicons, automatically converts them to multi-resolution Windows `.ico` formats, injects window constraints, and compiles the project.

### 3. Desktop Integration as a First-Class Citizen
A web page in a frame is not a desktop app. Henkei bridges the gap:
- **True Fullscreen Video**: Intercepts HTML5 fullscreen requests from video players (YouTube, Twitch, Netflix) and toggles native Windows borderless fullscreen states to hide the taskbar.
- **High-DPI Alignment**: Computes window coordinates using signed integers to ensure the window centers perfectly on High-DPI screens without wrapping off-screen.
- **User-Level Execution**: The installer runs under standard user permissions (`asInvoker`), writes files directly to `%LOCALAPPDATA%`, creates Desktop/Start Menu shortcuts, and registers in the Registry for standard Add/Remove Programs integration—all without triggering administrator (UAC) prompts.

---

## 🚀 Key Features

- ⚡ **Instant Incremental Builds**: Uses a persistent compiler target cache (`%TEMP%\henkei-target-cache`). While the first build compiles dependencies, subsequent builds compile in **under 2-3 seconds**.
- 🛠️ **Two Execution Modes**: Launch an interactive step-by-step console wizard, or run instantly using command-line flags.
- 🎛️ **Navigation Shortcuts**: Built-in client-side injections support standard navigation (e.g. `Alt + Left` to go back, `Alt + Right` to go forward).
- 🧹 **Graceful Uninstallation**: Includes a self-deleting uninstaller that purges registry records, application binaries, desktop shortcuts, and WebView2 cache directories.

---

## 📥 Requirements

- **OS**: Windows 10 or Windows 11
- **Tools**: Rust and Cargo compiler toolchain (to compile the CLI itself)

---

## 🔧 Installation & Setup

You can install `henkei` on your system using Cargo.

### Option 1: Install from crates.io (Once Published)
To install the latest release globally on your machine:
```powershell
cargo install henkei
```

### Option 2: Install from Local Source
If you are developing locally or compiled from source, install it using the path to the `henkei` crate:
```powershell
# Navigate to workspace root and run:
cargo install --path henkei
```
*Note: Make sure your Cargo bin directory (usually `%USERPROFILE%\.cargo\bin`) is in your system's `PATH` environment variable.*

### Option 3: Manual Local Compilation
If you prefer not to install it globally:
```powershell
# Compile the workspace in release mode
cargo build --release
```
The compiled CLI binary will be located at `./target/release/henkei.exe`.

---

## 📖 How to Use Henkei

### Mode A: The Guided Interactive Wizard (Recommended)
Simply launch `henkei` without any arguments. The CLI will open an interactive setup wizard in your console:
```powershell
henkei
```
**Interactive prompts:**
1. `🔗 Enter website URL`: Input the target URL (e.g. `https://news.ycombinator.com`).
2. `🏷️ Enter app name`: Enter a custom app name (or press `Enter` to auto-detect from the page title).
3. `↕️ Enter window width` / `↔️ Enter window height`: Custom launching dimensions (or press `Enter` for default `1200x800`).
4. `📁 Enter output directory`: Where to save the installer (or press `Enter` for `./build`).

---

### Mode B: Direct Command Line (For Automation & Scripting)
Run the CLI directly using flags to automate builds:

#### 1. Basic Compilation
Compile a website using its default scraped title and favicon:
```powershell
henkei https://news.ycombinator.com
```

#### 2. Specifying a Custom Name and Custom Directory
Override the auto-detected title and specify where the generated installer should go:
```powershell
henkei https://news.ycombinator.com --name "HackerNews" --output-dir "C:\Users\YourUsername\Desktop"
```

#### 3. Custom Resolutions (Great for video streaming apps)
```powershell
henkei https://youtube.com --name "YouTube" --width 1440 --height 900 --output-dir ./build
```

---

## 📦 What Happens Under the Hood?

When you run `henkei`, it executes the following steps:
1. **Scrapes HTML**: Crawls `<title>`, Open Graph properties, and `<link>` elements to parse names and icon locations.
2. **Converts Icons**: Downloads the highest resolution favicon, parses it, and writes it as a standard multi-resolution `.ico` file.
3. **Generates Code templates**: Generates custom Rust config code incorporating your dimensions, URL, and application name.
4. **Performs Cached Cargo Builds**: Compiles the WebView runner and installer templates, utilizing the persistent Cargo target cache to optimize compile speeds.
5. **Generates Installer**: Outputs `<SanitizedName>_installer.exe` containing the embedded WebView runner binary.
6. **Cleans Up**: Deletes temporary source files, leaving the workspace pristine.
