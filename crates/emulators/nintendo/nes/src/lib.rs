use std::ptr;

use emulators::Emulator;
use ffi::{CoreHandle, CoreInfo, CoreTable};
use nes::{bus::Bus, cpu::CpuRp2a03, rom::Rom, reset, tick};

struct NesEmu {
    cpu: CpuRp2a03,
    bus: Bus,
    rom_data: Vec<u8>,
    pad: base::frame::Gamepad,
    _pad2: base::frame::Gamepad,
}

impl Emulator for NesEmu {
    fn tick(&mut self) {
        // Sync gamepad to x-nes bus
        self.bus.pad1.a = self.pad.a;
        self.bus.pad1.b = self.pad.b;
        self.bus.pad1.select = self.pad.select;
        self.bus.pad1.start = self.pad.start;
        self.bus.pad1.up = self.pad.up;
        self.bus.pad1.down = self.pad.down;
        self.bus.pad1.left = self.pad.left;
        self.bus.pad1.right = self.pad.right;

        tick(&mut self.cpu, &mut self.bus);
    }

    fn frame_complete(&self) -> bool { self.bus.ppu.frame_complete }
    fn frame(&self) -> &[u8; 256 * 240] { &self.bus.ppu.frame }
    fn audio(&self) -> &[f32] { &self.bus.apu.audio_samples[..self.bus.apu.sample_count] }
    fn drain_audio(&mut self) { self.bus.apu.sample_count = 0; }
    fn pad1(&mut self) -> &mut base::frame::Gamepad { &mut self.pad }
    fn pad2(&mut self) -> &mut base::frame::Gamepad { &mut self._pad2 }

    fn reset(&mut self) {
        if let Some(rom) = Rom::new(&self.rom_data) {
            let mapper = rom.create_mapper();
            self.cpu = CpuRp2a03::new(0);
            self.bus = Bus::new(mapper);
            reset(&mut self.cpu, &mut self.bus);
        }
    }
    fn sample_rate(&self) -> u32 { 44100 }
}

struct CoreWrap(Box<dyn Emulator>);

extern "C" fn info() -> CoreInfo {
    CoreInfo {
        name: c"NES (x-nes)" as *const _ as *const u8,
        version: c"0.1.0" as *const _ as *const u8,
        extensions: c"nes,nez,unf" as *const _ as *const u8,
        magic_len: 4,
        magic: &0x4E45531Au32.to_le_bytes() as *const _ as *const u8,
    }
}

extern "C" fn create(data: *const u8, len: usize) -> CoreHandle {
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let rom = match Rom::new(slice) {
        Some(r) => r,
        None => return ptr::null_mut(),
    };
    let mut cpu = CpuRp2a03::new(0);
    let mut bus = Bus::new(rom.create_mapper());
    reset(&mut cpu, &mut bus);
    let emu = NesEmu {
        cpu,
        bus,
        rom_data: slice.to_vec(),
        pad: base::frame::Gamepad::new(),
        _pad2: base::frame::Gamepad::new(),
    };
    Box::into_raw(Box::new(CoreWrap(Box::new(emu)))) as CoreHandle
}

extern "C" fn destroy(handle: CoreHandle) {
    if !handle.is_null() { unsafe { drop(Box::from_raw(handle as *mut CoreWrap)); } }
}

extern "C" fn tick_(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut CoreWrap)).0.tick(); }
}

extern "C" fn frame_complete(handle: CoreHandle) -> bool {
    unsafe { (*(handle as *const CoreWrap)).0.frame_complete() }
}

extern "C" fn frame(handle: CoreHandle) -> *const u8 {
    unsafe { (*(handle as *const CoreWrap)).0.frame().as_ptr() }
}

extern "C" fn audio(handle: CoreHandle, out_len: &mut usize) -> *const f32 {
    let e = unsafe { &*(handle as *const CoreWrap) };
    let s = e.0.audio();
    *out_len = s.len();
    s.as_ptr()
}

extern "C" fn drain_audio(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut CoreWrap)).0.drain_audio(); }
}

extern "C" fn set_pad(handle: CoreHandle, byte: u8) {
    let e = unsafe { &mut *(handle as *mut CoreWrap) };
    let mut gp = base::frame::Gamepad::new();
    gp.from_byte(byte);
    *e.0.pad1() = gp;
}

extern "C" fn reset_(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut CoreWrap)).0.reset(); }
}

extern "C" fn sample_rate(_handle: CoreHandle) -> u32 { 44100 }

#[unsafe(no_mangle)]
pub static core_table: CoreTable = CoreTable {
    info, create, destroy,
    tick: tick_, frame_complete, frame, audio, drain_audio,
    set_pad, reset: reset_, sample_rate,
};
