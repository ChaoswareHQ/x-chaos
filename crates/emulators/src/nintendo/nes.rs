use base::frame::Gamepad;
use crate::Emulator;

use nes::{bus::Bus, cpu::CpuRp2a03, rom::Rom};
use nes::{reset, tick};

/// NES emulator implementing the [`Emulator`] trait.
///
/// Wraps the cycle-accurate `x-nes` core and maps its internal
/// state to the common emulator interface.
pub struct NesEmulator {
    cpu: CpuRp2a03,
    bus: Bus,
    /// Raw ROM data for reset
    rom_data: Vec<u8>,
    pad1: Gamepad,
    pad2: Gamepad,
}

impl NesEmulator {
    /// Load a NES ROM from raw data.
    ///
    /// Returns `None` if the data is not a valid iNES ROM.
    #[must_use]
    pub fn new(data: &[u8]) -> Option<Self> {
        let rom = Rom::new(data)?;
        let mut cpu = CpuRp2a03::new(0);
        let mut bus = Bus::new(rom.create_mapper());
        reset(&mut cpu, &mut bus);

        Some(Self {
            cpu,
            bus,
            rom_data: data.to_vec(),
            pad1: Gamepad::new(),
            pad2: Gamepad::new(),
        })
    }

    /// Set the audio sample rate.
    pub fn set_sample_rate(&mut self, rate: f64) {
        self.bus.apu.set_sample_rate(rate);
    }

    /// Access the internal bus (for advanced debug / save states).
    pub fn bus(&self) -> &Bus {
        &self.bus
    }
}

impl Emulator for NesEmulator {
    fn tick(&mut self) {
        // Sync input before each tick
        self.bus.pad1.a = self.pad1.a;
        self.bus.pad1.b = self.pad1.b;
        self.bus.pad1.select = self.pad1.select;
        self.bus.pad1.start = self.pad1.start;
        self.bus.pad1.up = self.pad1.up;
        self.bus.pad1.down = self.pad1.down;
        self.bus.pad1.left = self.pad1.left;
        self.bus.pad1.right = self.pad1.right;

        tick(&mut self.cpu, &mut self.bus);
    }

    fn frame_complete(&self) -> bool {
        self.bus.ppu.frame_complete
    }

    fn frame(&self) -> &[u8; 256 * 240] {
        &self.bus.ppu.frame
    }

    fn audio(&self) -> &[f32] {
        &self.bus.apu.audio_samples[..self.bus.apu.sample_count]
    }

    fn drain_audio(&mut self) {
        self.bus.apu.sample_count = 0;
    }

    fn pad1(&mut self) -> &mut Gamepad {
        &mut self.pad1
    }

    fn pad2(&mut self) -> &mut Gamepad {
        &mut self.pad2
    }

    fn reset(&mut self) {
        if let Some(rom) = Rom::new(&self.rom_data) {
            let mapper = rom.create_mapper();
            self.cpu = CpuRp2a03::new(0);
            self.bus = Bus::new(mapper);
            reset(&mut self.cpu, &mut self.bus);
        }
    }

    fn sample_rate(&self) -> u32 {
        44100
    }
}
