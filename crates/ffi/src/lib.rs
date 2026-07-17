/// C ABI for emulator core shared libraries.
///
/// Every core `.dll` exports a `core_table` symbol — x-chaos loads
/// them at runtime via `libloading`.

pub const FRAME_W: u32 = 256;
pub const FRAME_H: u32 = 240;

#[repr(C)]
pub struct CoreInfo {
    pub name: *const u8,
    pub version: *const u8,
    pub extensions: *const u8,
    pub magic_len: usize,
    pub magic: *const u8,
}

pub type CoreHandle = *mut core::ffi::c_void;

/// Function table exported by every core .dll as `core_table`.
#[repr(C)]
pub struct CoreTable {
    pub info: extern "C" fn() -> CoreInfo,
    pub create: extern "C" fn(*const u8, usize) -> CoreHandle,
    pub destroy: extern "C" fn(CoreHandle),
    pub tick: extern "C" fn(CoreHandle),
    pub frame_complete: extern "C" fn(CoreHandle) -> bool,
    pub frame: extern "C" fn(CoreHandle) -> *const u8,
    pub audio: extern "C" fn(CoreHandle, &mut usize) -> *const f32,
    pub drain_audio: extern "C" fn(CoreHandle),
    pub set_pad: extern "C" fn(CoreHandle, u8),
    pub reset: extern "C" fn(CoreHandle),
    pub frame_ack: extern "C" fn(CoreHandle),
    pub sample_rate: extern "C" fn(CoreHandle) -> u32,
    /// Set the audio sample rate so the APU generates samples
    /// matching the audio device's rate.
    pub set_sample_rate: extern "C" fn(CoreHandle, f64),
}
