#[allow(unused_imports)]
use core::arch::asm;

pub struct Bits;

impl Bits {
    read_csr_as_usize!(0x347);
    write_csr_as!(0x347);
    set!(0x347);
    clear!(0x347);
}
