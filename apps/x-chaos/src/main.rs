mod core_loader;

use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use gfx::desktop::{DesktopSurface, Event};
use gfx::palette::Palette;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;

const NES_W: u32 = 256;
const NES_H: u32 = 240;
const SCALE: u32 = 3;
const FRAME_DUR: Duration = Duration::from_nanos(16_639_000);

struct AudioOut {
    #[allow(dead_code)]
    stream: cpal::Stream,
    tx: ringbuf::CachingProd<Arc<HeapRb<f32>>>,
    sr: u32,
    hold: Arc<AtomicU32>,
}

fn init_audio() -> Option<AudioOut> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let supported = device.default_output_config().ok()?;
    let sr: u32 = supported.sample_rate();
    let ch = supported.channels() as usize;
    eprintln!("Audio: {sr} Hz, {ch} ch");

    let rb = HeapRb::<f32>::new(65536);
    let (mut prod, mut cons) = rb.split();
    let hold = Arc::new(AtomicU32::new(0));
    let hold_cb = hold.clone();

    for _ in 0..(sr as f64 / 60.0 * 4.0) as usize {
        let _ = prod.try_push(0.0);
    }

    let config: cpal::StreamConfig = supported.into();
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(ch) {
                    let s = cons.try_pop().unwrap_or_else(|| {
                        f32::from_bits(hold_cb.load(Ordering::Relaxed))
                    });
                    hold_cb.store(s.to_bits(), Ordering::Relaxed);
                    for sample in frame.iter_mut() {
                        *sample = s;
                    }
                }
            },
            |e| eprintln!("audio error: {e}"),
            None,
        )
        .ok()?;
    stream.play().ok()?;
    Some(AudioOut { stream, tx: prod, sr, hold })
}

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

    let mut audio = init_audio();
    if let Some(ref a) = audio {
        (core.table.set_sample_rate)(handle, a.sr as f64);
    }

    let palette = Palette::nes();
    let mut last = Instant::now();
    let mut raw_frame = [0u8; (NES_W * NES_H) as usize];
    let mut pad = base::frame::Gamepad::new();

    DesktopSurface::run("x-chaos", NES_W * SCALE, NES_H * SCALE, move |surface, event| {
        match event {
            Event::Close => return false,

            Event::Key(ke) => {
                use gfx::winit::keyboard::{KeyCode, PhysicalKey};
                let pressed = ke.state.is_pressed();
                match ke.physical_key {
                    PhysicalKey::Code(KeyCode::KeyZ) => pad.a = pressed,
                    PhysicalKey::Code(KeyCode::KeyX) => pad.b = pressed,
                    PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => {
                        pad.select = pressed
                    }
                    PhysicalKey::Code(KeyCode::Enter) => pad.start = pressed,
                    PhysicalKey::Code(KeyCode::ArrowUp) => pad.up = pressed,
                    PhysicalKey::Code(KeyCode::ArrowDown) => pad.down = pressed,
                    PhysicalKey::Code(KeyCode::ArrowLeft) => pad.left = pressed,
                    PhysicalKey::Code(KeyCode::ArrowRight) => pad.right = pressed,
                    _ => return true,
                }
                (core.table.set_pad)(handle, pad.to_byte());
            }

            Event::Draw => {
                let now = Instant::now();
                if now - last < FRAME_DUR {
                    return true;
                }
                last = now;

                while !(core.table.frame_complete)(handle) {
                    (core.table.tick)(handle);
                }

                let fptr = (core.table.frame)(handle);
                if !fptr.is_null() {
                    let frame = unsafe {
                        std::slice::from_raw_parts(fptr, (NES_W * NES_H) as usize)
                    };
                    raw_frame.copy_from_slice(frame);
                }

                (core.table.frame_ack)(handle);

                // Audio
                let mut audio_len = 0;
                let audio_ptr = (core.table.audio)(handle, &mut audio_len);
                if audio_len > 0 {
                    let samples = unsafe { std::slice::from_raw_parts(audio_ptr, audio_len) };
                    if let Some(ref mut a) = audio {
                        let pushed = a.tx.push_slice(samples);
                        if pushed > 0 {
                            a.hold.store(samples[pushed - 1].to_bits(), Ordering::Relaxed);
                        }
                        if pushed < audio_len && pushed == 0 {
                            eprintln!("audio buffer full");
                        }
                    }
                }
                (core.table.drain_audio)(handle);

                // Render
                let (dw, dh) = surface.size();
                let mut buf = vec![0u32; (dw * dh) as usize];
                palette.fill_frame(&raw_frame, &mut buf, dw, dh);
                surface.present(&buf, dw, dh);
            }

            Event::Resize(_, _) => {}
        }

        true
    });
}
