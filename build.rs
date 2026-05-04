// Embeds the application icon (and a few common version-info fields) into
// the resulting .exe via the Windows resource compiler. Only runs when
// targeting Windows; on other targets this is a no-op.

fn main() {
    println!("cargo:rerun-if-changed=assets/cpu-mem-overlay.ico");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/cpu-mem-overlay.ico");
        res.set("FileDescription", "CPU/Memory overlay");
        res.set("ProductName", "cpu-mem-overlay");
        res.compile()
            .expect("failed to compile Windows resource (icon + version info)");
    }
}
