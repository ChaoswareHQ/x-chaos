use std::path::PathBuf;

use ffi::CoreTable;

pub struct CoreDll {
    #[allow(dead_code)]
    pub name: String,
    pub table: &'static CoreTable,
    #[allow(dead_code)]
    lib: libloading::Library,
}

/// Scan `cores/` next to the executable for all `.dll` files.
///
/// # Safety
///
/// Each `.dll` must export a valid `core_table` symbol.
pub unsafe fn scan_cores() -> Vec<CoreDll> {
    let mut cores = vec![];

    if let Some(dir) = cores_dir() {
        if dir.exists() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("dll") {
                    continue;
                }
                if let Some(core) = unsafe { load(&path) } {
                    cores.push(core);
                }
            }
        }
    }

    cores
}

/// Load a single core from a `.dll` path.
///
/// # Safety
///
/// The `.dll` must export a valid `core_table` symbol.
pub unsafe fn load(path: &PathBuf) -> Option<CoreDll> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();
    // SAFETY: The caller guarantees the .dll exports a valid core_table.
    let lib = unsafe { libloading::Library::new(path).ok()? };
    let sym: libloading::Symbol<*const CoreTable> =
        unsafe { lib.get(b"core_table").ok()? };
    let table = unsafe { &**sym };
    println!("Loaded core: {} ({})", name, path.display());
    Some(CoreDll { name, table, lib })
}

fn cores_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|p| p.join("cores"))
}
