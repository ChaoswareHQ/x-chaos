/// NES-style gamepad state.
#[derive(Clone, Copy, Default)]
pub struct Gamepad {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Gamepad {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            a: false, b: false, select: false, start: false,
            up: false, down: false, left: false, right: false,
        }
    }

    #[must_use]
    pub fn to_byte(&self) -> u8 {
        let mut b = 0u8;
        if self.a { b |= 0x01; }
        if self.b { b |= 0x02; }
        if self.select { b |= 0x04; }
        if self.start { b |= 0x08; }
        if self.up { b |= 0x10; }
        if self.down { b |= 0x20; }
        if self.left { b |= 0x40; }
        if self.right { b |= 0x80; }
        b
    }

    pub fn from_byte(&mut self, b: u8) {
        self.a = b & 0x01 != 0;
        self.b = b & 0x02 != 0;
        self.select = b & 0x04 != 0;
        self.start = b & 0x08 != 0;
        self.up = b & 0x10 != 0;
        self.down = b & 0x20 != 0;
        self.left = b & 0x40 != 0;
        self.right = b & 0x80 != 0;
    }
}
