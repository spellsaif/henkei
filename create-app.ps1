# Henkei App Creation Wizard
Clear-Host

Write-Host "==================================================" -ForegroundColor Green
Write-Host "         Henkei Web-to-Desktop Wizard             " -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green
Write-Host "This wizard will help you package any website into"
Write-Host "a highly optimized Windows Desktop App installer."
Write-Host ""

# 1. Ask for Target URL
$url = Read-Host "🔗 Enter website URL (e.g. https://news.ycombinator.com)"
if ([string]::IsNullOrWhiteSpace($url)) {
    Write-Host "❌ Error: URL is required!" -ForegroundColor Red
    exit 1
}

# Ensure URL has protocol
if (-not ($url.StartsWith("http://") -or $url.StartsWith("https://"))) {
    $url = "https://" + $url
}

# 2. Ask for Custom Name
$name = Read-Host "🏷️ Enter app name (optional, press Enter to auto-detect)"
$name = $name.Trim()

# 3. Ask for Custom Resolution
$width = Read-Host "↕️ Enter window width (optional, press Enter for 1200)"
$width = $width.Trim()
$height = Read-Host "↔️ Enter window height (optional, press Enter for 800)"
$height = $height.Trim()

# 4. Ask for Output Directory
$outDir = Read-Host "📁 Enter output directory (optional, press Enter for './build')"
$outDir = $outDir.Trim()

# Build the CLI command
$exePath = Join-Path $PSScriptRoot "target\release\henkei.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "⚠️ CLI binary not found. Compiling henkei first..." -ForegroundColor Yellow
    & cargo build --release
    if (-not (Test-Path $exePath)) {
        Write-Host "❌ Error: Failed to compile henkei. Please make sure Rust is installed." -ForegroundColor Red
        exit 1
    }
}

$cmdArgs = @($url)

if (-not [string]::IsNullOrWhiteSpace($name)) {
    $cmdArgs += "--name", $name
}

if (-not [string]::IsNullOrWhiteSpace($outDir)) {
    $cmdArgs += "--output-dir", $outDir
    $outPath = $outDir
} else {
    $cmdArgs += "--output-dir", "./build"
    $outPath = Join-Path $PSScriptRoot "build"
}

if (-not [string]::IsNullOrWhiteSpace($width)) {
    $cmdArgs += "--width", $width
}
if (-not [string]::IsNullOrWhiteSpace($height)) {
    $cmdArgs += "--height", $height
}

Write-Host ""
Write-Host "🚀 Running Henkei Compiler..." -ForegroundColor Cyan
Write-Host ""

# Run compiler
& $exePath $cmdArgs

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "🎉 Success! Application installer created." -ForegroundColor Green
    
    # Resolve absolute path for output folder and open it in explorer
    $absOutPath = Resolve-Path $outPath -ErrorAction SilentlyContinue
    if ($absOutPath) {
        Write-Host "📂 Opening output folder: $absOutPath" -ForegroundColor Gray
        explorer.exe $absOutPath.Path
    }
} else {
    Write-Host ""
    Write-Host "❌ Compilation failed." -ForegroundColor Red
}
