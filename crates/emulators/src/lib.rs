use base::frame::Gamepad;

pub const FRAME_W: u32 = 256;
pub const FRAME_H: u32 = 240;

/// Common interface that all emulator cores implement.
pub trait Emulator {
    fn tick(&mut self);
    fn frame_complete(&self) -> bool;
    fn frame(&self) -> &[u8; (FRAME_W * FRAME_H) as usize];
    fn audio(&self) -> &[f32];
    fn drain_audio(&mut self);
    fn pad1(&mut self) -> &mut Gamepad;
    fn pad2(&mut self) -> &mut Gamepad;
    fn reset(&mut self);
    fn sample_rate(&self) -> u32;
}
