fn main() {
    #[cfg(target_os = "windows")]
    {
        if std::path::Path::new("app.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("app.ico");
            let _ = res.compile();
        }
    }
}
