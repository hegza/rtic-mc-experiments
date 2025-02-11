macro_rules! read_csr {
    ($csr_number:literal) => {
        /// Reads the CSR
        #[inline]
        unsafe fn _read() -> usize {
            let r: usize;
            core::arch::asm!(concat!("csrrs {0}, ", stringify!($csr_number), ", x0"), out(reg) r);
            r
        }
    };
}

macro_rules! write_csr {
    ($csr_number:literal) => {
        /// Writes the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _write(bits: usize) -> usize {
            let v;
            core::arch::asm!(concat!("csrrw {1}, ", stringify!($csr_number), ", {0}"), in(reg) bits, out(reg) v);
            v
        }
    };
}

macro_rules! write_csr_as {
    ($csr_number:literal) => {
        write_csr!($csr_number);

        /// Writes the CSR and returns the old value
        #[inline]
        pub unsafe fn write(bits: usize) -> usize {
            unsafe { Self::_write(bits) }
        }
    };
}

macro_rules! clear {
    ($csr_number:literal) => {
        /// Clear the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _clear(bits: usize) {
            core::arch::asm!(concat!("csrrc x0, ", stringify!($csr_number), ", {0}"), in(reg) bits)
        }
    };
}

macro_rules! set {
    ($csr_number:literal) => {
        /// Set the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _set(bits: usize) {
            core::arch::asm!(concat!("csrrs x0, ", stringify!($csr_number), ", {0}"), in(reg) bits)
        }
    };
}

macro_rules! read_csr_as_usize {
    ($csr_number:literal) => {
        read_csr!($csr_number);

        /// Reads the CSR
        #[inline]
        pub unsafe fn read() -> usize {
            unsafe { Self::_read() }
        }
    };
}
