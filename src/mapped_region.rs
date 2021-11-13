use libc::{c_void, size_t};
use std::borrow::Borrow;
use std::ops::{Drop, Index};
use std::ptr;

#[cfg(target_os = "macos")]
const MAP_FAILED: *mut c_void = (!0usize) as *mut c_void;

pub struct MappedRegion {
    addr: *mut c_void,
    len: size_t,
}

impl MappedRegion {
    pub fn allocate(size: usize) -> Result<Self, &'static str> {
        use libc::{MAP_ANON, MAP_JIT, MAP_PRIVATE, PROT_READ};
        let memory;
        unsafe {
            memory = libc::mmap(
                ptr::null_mut(),
                size,
                PROT_READ,
                MAP_PRIVATE | MAP_ANON | MAP_JIT,
                -1,
                0,
            );
        }

        if memory == MAP_FAILED {
            return Err("Could not allocate page");
        }

        Ok(MappedRegion {
            addr: memory,
            len: size,
        })
    }

    pub fn addr(&self) -> *const c_void {
        self.addr
    }

    pub fn addr_mut(&self) -> *mut c_void {
        self.addr
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<I> Index<I> for MappedRegion
where
    I: std::slice::SliceIndex<[u8]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        unsafe { &std::slice::from_raw_parts(self.addr as *const u8, self.len)[index] }
    }
}

impl Borrow<[u8]> for MappedRegion {
    fn borrow(&self) -> &[u8] {
        &self[..]
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        println!("Unmapping that page...");
        unsafe {
            libc::munmap(self.addr, self.len);
        }
        self.addr = std::ptr::null_mut();
        self.len = 0;
        println!("dropped!");
    }
}
