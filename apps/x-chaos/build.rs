fn main() {
    // Copy the built NES core -> cores/nes.dll next to executable
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target = std::path::Path::new(&out_dir);
    let build_dir = target
        .ancestors()
        .find(|p| p.join("deps").exists())
        .unwrap();

    let deps = build_dir.join("deps");
    if deps.exists() {
        for entry in std::fs::read_dir(&deps).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with("nes.dll") || (name.starts_with("nes-") && name.ends_with(".dll")) {
                let cores_dir = build_dir.join("cores");
                std::fs::create_dir_all(&cores_dir).unwrap();
                std::fs::copy(&path, cores_dir.join("nes.dll")).unwrap();
                break;
            }
        }
    }
}
