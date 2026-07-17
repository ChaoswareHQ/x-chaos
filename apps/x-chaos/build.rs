use std::path::Path;

fn main() {
    // Copy the built NES core to cores/nes.dll next to the executable.
    // Check both the deps/ directory and the target root.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out_dir);

    // Walk up from OUT_DIR to find the build root (where x-chaos.exe ends up)
    let build_root = out.ancestors()
        .find(|p| p.join("build.rs").exists() || p.ends_with("release") || p.ends_with("debug"))
        .unwrap_or_else(|| out);

    // The cdylib might be in build_root itself or build_root/deps/
    let candidates = [
        build_root.join("nes.dll"),
        build_root.join("deps").join("nes.dll"),
    ];

    for src in &candidates {
        if src.exists() {
            let cores = build_root.join("cores");
            std::fs::create_dir_all(&cores).unwrap();
            std::fs::copy(src, cores.join("nes.dll")).unwrap();
            println!("cargo:warning=NES core copied");
            return;
        }
    }
}
