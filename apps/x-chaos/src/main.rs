// x-chaos — Multi-system emulator frontend
//
// Structure:
//   main.rs        Entry point, CLI argument parsing
//   app.rs         Application state and event loop
//   ui/            Slint UI definitions

mod app;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("Usage: x-chaos <rom-file> [system]");
    }

    let rom_path = &args[1];
    let system = args.get(2).map(|s| s.as_str()).unwrap_or("nes");

    // Detect system by file extension if not specified
    let system = if system == "nes" || rom_path.ends_with(".nes") {
        "nes"
    } else {
        anyhow::bail!("Unsupported system: {system}");
    };

    let data = std::fs::read(rom_path)
        .with_context(|| format!("Failed to read ROM: {rom_path}"))?;

    // Create the emulator
    let emu: Box<dyn emulators::Emulator> = match system {
        "nes" => {
            let nes = emulators::nintendo::NesEmulator::new(&data)
                .context("Invalid NES ROM")?;
            Box::new(nes)
        }
        _ => unreachable!(),
    };

    // Run the app
    app::App::run(emu)?;

    Ok(())
}
