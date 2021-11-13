extern crate libc;

use libc::{c_void, size_t};
use std::borrow::{Borrow, BorrowMut};
use std::ops::{Drop, Index, IndexMut};
use std::ptr;
use std::slice::SliceIndex;

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

pub struct WritableRegion {
    region: MappedRegion,
}

impl WritableRegion {
    pub fn from(region: MappedRegion) -> Result<Self, &'static str> {
        use libc::{PROT_READ, PROT_WRITE};

        unsafe {
            if libc::mprotect(region.addr, region.len, PROT_READ | PROT_WRITE) < 0 {
                return Err("could not change protection");
            }
        }

        Ok(Self { region })
    }
}

impl<I> Index<I> for WritableRegion
where
    I: SliceIndex<[u8]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        unsafe {
            &std::slice::from_raw_parts(self.region.addr as *const u8, self.region.len)[index]
        }
    }
}

impl<I> IndexMut<I> for WritableRegion
where
    I: SliceIndex<[u8]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        unsafe {
            &mut std::slice::from_raw_parts_mut(self.region.addr as *mut u8, self.region.len)[index]
        }
    }
}

impl Borrow<[u8]> for WritableRegion {
    fn borrow(&self) -> &[u8] {
        &self.region[..]
    }
}

impl BorrowMut<[u8]> for WritableRegion {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self[..]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_gives_a_valid_address() {
        let size = 4096;
        match MappedRegion::allocate(size) {
            Ok(region) => {
                assert_eq!(size, region.len());
                assert_ne!(region.addr(), ptr::null_mut());
                assert_ne!(region.addr(), MAP_FAILED);
            }
            Err(err) => {
                panic!("could not map area: {}", err);
            }
        }
    }
}
