mod ffi {
    use libc::{c_int, c_long};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

/* Disable ASLR */
pub fn disable_aslr() -> () {
    unsafe {
        let old = ffi::personality(0xffffffff);
        ffi::personality((old | 0x0040000) as i64);
    }
}

