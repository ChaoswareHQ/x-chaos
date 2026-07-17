#[cfg(feature = "gpu")]
pub mod desktop;

pub mod frame;
pub mod palette;

#[cfg(feature = "gpu")]
pub use winit;

pub const FRAME_W: u32 = 256;
pub const FRAME_H: u32 = 240;
