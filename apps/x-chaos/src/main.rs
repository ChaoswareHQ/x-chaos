mod core_loader;

use std::time::{Duration, Instant};

use gfx::desktop::{DesktopSurface, Event};
use gfx::palette::Palette;

const NES_W: u32 = 256;
const NES_H: u32 = 240;
const SCALE: u32 = 3;
const NES_FRAME_NS: u64 = 16_639_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: x-chaos <rom.nes>");
        eprintln!("       Drag a ROM file onto the executable");
        std::process::exit(1);
    }

    let rom_data = std::fs::read(&args[1]).expect("failed to read ROM");

    let cores = unsafe { core_loader::scan_cores() };
    let core = cores.first().expect("No cores found in cores/ directory");
    let handle = (core.table.create)(rom_data.as_ptr(), rom_data.len());
    assert!(!handle.is_null(), "Core rejected the ROM");

    let palette = Palette::nes();
    let mut acc = Duration::new(0, 0);
    let mut last = Instant::now();
    let frame_dur = Duration::from_nanos(NES_FRAME_NS);

    DesktopSurface::run("x-chaos", NES_W * SCALE, NES_H * SCALE, move |surface, event| {
        match event {
            Event::Close => return false,

            Event::Key(ke) => {
                use base::frame::Gamepad;
                use gfx::winit::keyboard::{KeyCode, PhysicalKey};

                let mut gp = Gamepad::new();
                match ke.physical_key {
                    PhysicalKey::Code(KeyCode::KeyZ) => gp.a = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::KeyX) => gp.b = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => {
                        gp.select = ke.state.is_pressed()
                    }
                    PhysicalKey::Code(KeyCode::Enter) => gp.start = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::ArrowUp) => gp.up = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::ArrowDown) => gp.down = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::ArrowLeft) => gp.left = ke.state.is_pressed(),
                    PhysicalKey::Code(KeyCode::ArrowRight) => gp.right = ke.state.is_pressed(),
                    _ => return true,
                }
                (core.table.set_pad)(handle, gp.to_byte());
            }

            Event::Draw => {
                let now = Instant::now();
                acc += now - last;
                last = now;
                if acc > Duration::from_millis(100) {
                    acc = Duration::from_millis(100);
                }

                while acc >= frame_dur {
                    while !(core.table.frame_complete)(handle) {
                        (core.table.tick)(handle);
                    }
                    acc -= frame_dur;

                    // Audio
                    let mut audio_len = 0;
                    let audio_ptr = (core.table.audio)(handle, &mut audio_len);
                    if audio_len > 0 {
                        let _ = unsafe { std::slice::from_raw_parts(audio_ptr, audio_len) };
                    }
                    (core.table.drain_audio)(handle);
                }

                // Render
                let (dw, dh) = surface.size();
                let buf = surface.buffer_mut();
                if !buf.is_empty() {
                    let frame_ptr = (core.table.frame)(handle);
                    if !frame_ptr.is_null() {
                        let frame = unsafe {
                            std::slice::from_raw_parts(frame_ptr, (NES_W * NES_H) as usize)
                        };
                        palette.fill_frame(frame, buf, dw.max(1), dh.max(1));
                    }
                }
            }

            Event::Resize(_, _) => {}
        }

        true
    });
}
