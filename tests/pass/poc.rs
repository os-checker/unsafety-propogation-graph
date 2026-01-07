#![allow(dead_code)]
#![feature(register_tool)]
#![register_tool(rapx)]

/// Record secret buffer with its size.
struct SecretRegion {
    buffer: *mut u32,
    len: usize,
}

impl SecretRegion {
    #[rapx::requires(ValidPtr(v), InitializedInLen(l))]
    pub unsafe fn from(v: *mut u32, l: usize) -> Self {
        SecretRegion { buffer: v, len: l }
    }

    #[rapx::requires(hazard.InitializedInLen(l))]
    pub unsafe fn set_len(&mut self, l: usize) {
        self.len = l;
    }

    #[rapx::requires(ValidPtr(ptr), ValidPtr(self.buffer, offset))]
    pub unsafe fn xor_secret_region(&self, ptr: *mut u32, offset: isize) -> u32 {
        let mut src_value = ptr.read();
        let secret_ptr = self.buffer;
        let secret_region_ptr = secret_ptr.offset(offset);
        let secret_value = secret_region_ptr.read();
        src_value ^= secret_value;
        src_value
    }
}

fn f() {
    let v = vec![0xDEADBEEFu32, 0xCAFEBABE, 0x12345678];
    let mut data = [0x11111111u32, 0x22222222, 0x33333333];
    let (p, l, _c) = v.into_raw_parts();
    let mut s = unsafe { SecretRegion::from(p, 0) };
    unsafe {
        s.set_len(l);
        s.xor_secret_region(data.as_mut_ptr(), 0);
    }
}
