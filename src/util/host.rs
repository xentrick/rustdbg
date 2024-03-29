use std::mem;

#[derive(Copy, Clone, Debug)]
enum Endianness {
    Big, Little,
}

impl Endianness {
    fn target() -> Self {
        #[cfg(target_endian = "big")]
        {
            Endianness::Big
        }
        #[cfg(not(target_endian = "big"))]
        {
            Endianness::Little
        }
    }
}

pub fn wordsize() {
    std::mem::size_of::<usize>()
}

pub fn endianness() {
    Endianness::target()
}
