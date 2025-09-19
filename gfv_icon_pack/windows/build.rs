// build.rs — embeds Windows icon for taskbar & exe
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/windows/app_icon.ico");
        res.compile().unwrap();
    }
}
