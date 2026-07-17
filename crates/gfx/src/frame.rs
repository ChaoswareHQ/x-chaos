use crate::FRAME_W;
use crate::FRAME_H;

/// A borrowed 256×240 palette-indexed frame buffer.
pub struct Frame<'a> {
    data: &'a [u8; (FRAME_W * FRAME_H) as usize],
}

impl<'a> Frame<'a> {
    #[inline(always)]
    #[must_use]
    pub fn new(data: &'a [u8; (FRAME_W * FRAME_H) as usize]) -> Self {
        Self { data }
    }

    #[inline(always)]
    #[must_use]
    pub const fn width(&self) -> u32 { FRAME_W }

    #[inline(always)]
    #[must_use]
    pub const fn height(&self) -> u32 { FRAME_H }

    #[inline(always)]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] { self.data }

    #[inline(always)]
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.data[y as usize * FRAME_W as usize + x as usize]
    }
}
