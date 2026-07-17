use std::ptr;

use base::frame::Gamepad;
use emulators::Emulator;
use ffi::{CoreHandle, CoreInfo, CoreTable};
use nes::{bus::Bus, cpu::CpuRp2a03, rom::Rom, reset, tick};

struct NesEmu {
    cpu: CpuRp2a03,
    bus: Bus,
    rom_data: Vec<u8>,
    pad: Gamepad,
}

impl NesEmu {
    fn tick(&mut self) {
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

    fn frame_ready(&self) -> bool { self.bus.ppu.frame_complete }
    fn frame_ack(&mut self) { self.bus.ppu.frame_complete = false; }
}

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
    Box::into_raw(Box::new(NesEmu {
        cpu, bus, rom_data: slice.to_vec(), pad: Gamepad::new(),
    })) as CoreHandle
}

extern "C" fn destroy(handle: CoreHandle) {
    if !handle.is_null() { unsafe { drop(Box::from_raw(handle as *mut NesEmu)); } }
}

extern "C" fn tick_(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut NesEmu)).tick(); }
}

extern "C" fn frame_complete(handle: CoreHandle) -> bool {
    unsafe { (*(handle as *const NesEmu)).frame_ready() }
}

extern "C" fn frame_ack(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut NesEmu)).frame_ack(); }
}

extern "C" fn frame(handle: CoreHandle) -> *const u8 {
    unsafe { (*(handle as *const NesEmu)).bus.ppu.frame.as_ptr() }
}

extern "C" fn audio(handle: CoreHandle, out_len: &mut usize) -> *const f32 {
    let e = unsafe { &*(handle as *const NesEmu) };
    *out_len = e.bus.apu.sample_count;
    e.bus.apu.audio_samples.as_ptr()
}

extern "C" fn drain_audio(handle: CoreHandle) {
    unsafe { (&mut *(handle as *mut NesEmu)).bus.apu.sample_count = 0; }
}

extern "C" fn set_pad(handle: CoreHandle, byte: u8) {
    let e = unsafe { &mut *(handle as *mut NesEmu) };
    let mut gp = Gamepad::new();
    gp.from_byte(byte);
    e.pad = gp;
}

extern "C" fn reset_(handle: CoreHandle) {
    let e = unsafe { &mut *(handle as *mut NesEmu) };
    if let Some(rom) = Rom::new(&e.rom_data) {
        let mapper = rom.create_mapper();
        e.cpu = CpuRp2a03::new(0);
        e.bus = Bus::new(mapper);
        reset(&mut e.cpu, &mut e.bus);
    }
}

extern "C" fn sample_rate(_handle: CoreHandle) -> u32 { 44100 }

extern "C" fn set_sample_rate(handle: CoreHandle, rate: f64) {
    unsafe { (&mut *(handle as *mut NesEmu)).bus.apu.set_sample_rate(rate); }
}

#[unsafe(no_mangle)]
pub static core_table: CoreTable = CoreTable {
    info, create, destroy,
    tick: tick_, frame_complete, frame, audio, drain_audio,
    set_pad, reset: reset_, frame_ack, sample_rate, set_sample_rate,
};
