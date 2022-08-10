#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKey {
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

impl PartialEq<u8> for ChipKey {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}
