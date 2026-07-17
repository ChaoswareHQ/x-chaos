pub mod nintendo;

use base::frame::Gamepad;

/// NES frame dimensions used by the [`Emulator`] trait.
pub const FRAME_W: u32 = 256;
pub const FRAME_H: u32 = 240;

/// Common interface that all emulator cores implement.
///
/// The host loop calls [`Self::tick()`] in a tight loop, checks
/// [`Self::frame_complete()`], then drains audio and passes the
/// frame + palette to the gfx crate.
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
