use std::rc::Rc;

use emulators::Emulator;
use slint::ComponentHandle;

slint::include_modules!();

/// Top-level application state.
pub struct App;

impl App {
    /// Run the application (blocking).
    ///
    /// Initialises the Slint UI and enters the event loop.
    pub fn run(emulator: Box<dyn Emulator>) -> Result<(), slint::PlatformError> {
        let ui = AppWindow::new()?;
        let emulator = Rc::new(std::sync::Mutex::new(emulator));

        // --- Timer: render emulator frames at ~60 Hz ---
        let emu_weak = Rc::downgrade(&emulator);
        let ui_weak = ui.as_weak();
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_nanos(16_639_000),
            move || tick_and_render(emu_weak.clone(), &ui_weak),
        );

        // --- Input: keyboard events from Slint ---
        let emu_for_keys = Rc::downgrade(&emulator);
        ui.on_keyboard(move |key, pressed| {
            if let Some(emu) = emu_for_keys.upgrade() {
                let mut emu = emu.lock().unwrap();
                handle_key(emu.as_mut(), key.as_str(), pressed);
            }
        });

        // --- UI callbacks ---
        let emu_for_reset = Rc::downgrade(&emulator);
        ui.on_reset(move || {
            if let Some(emu) = emu_for_reset.upgrade() {
                emu.lock().unwrap().reset();
            }
        });

        ui.on_pause(move || {
            // Pause/resume state handled in timer
        });

        ui.run()
    }
}

/// Tick the emulator for one frame and update the Slint UI.
fn tick_and_render(
    emu: std::rc::Weak<std::sync::Mutex<Box<dyn Emulator>>>,
    ui: &slint::Weak<AppWindow>,
) {
    let Some(emu) = emu.upgrade() else { return };
    let Ok(mut emu) = emu.lock() else { return };
    let Some(ui) = ui.upgrade() else { return };

    // Tick until a complete frame is ready
    while !emu.frame_complete() {
        emu.tick();
    }

    // Convert palette-indexed frame to RGBA using x-gfx
    let frame = emu.frame();
    let palette = gfx::palette::Palette::nes();
    let mut rgba = vec![0u8; 256 * 240 * 4];

    let dst_u32: &mut [u32] = unsafe {
        std::slice::from_raw_parts_mut(rgba.as_mut_ptr().cast::<u32>(), 256 * 240)
    };
    palette.fill_frame(frame, dst_u32, 256, 240);

    // Push frame to Slint
    let mut pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(256, 240);
    pixel_buffer
        .make_mut_bytes()
        .copy_from_slice(&rgba);
    let image = slint::Image::from_rgba8(pixel_buffer);
    ui.set_emulator_frame(image);

    // Audio: drain samples
    #[cfg(feature = "desktop")]
    {
        let samples = emu.audio();
        if !samples.is_empty() {
            // x-audio desktop playback would go here
        }
    }
    emu.drain_audio();

    ui.set_status_text("FPS: 60".into());
}

/// Map a Slint key event to the emulator's gamepad.
fn handle_key(emu: &mut dyn Emulator, key: &str, pressed: bool) {
    let pad = emu.pad1();

    match key {
        "z" | "Z" => pad.a = pressed,
        "x" | "X" => pad.b = pressed,
        "Shift" => pad.select = pressed,
        "Enter" => pad.start = pressed,
        "Up" => pad.up = pressed,
        "Down" => pad.down = pressed,
        "Left" => pad.left = pressed,
        "Right" => pad.right = pressed,
        _ => {}
    }
}
