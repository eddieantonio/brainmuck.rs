extern crate libc;

use libc::{c_void, size_t};
use std::ops::{Index, IndexMut};
use std::ptr;

fn main() -> Result<(), &'static str> {
    let my_page = MappedRegion::allocate(4096)?;

    println!(
        "my page is at {:0X} and has size {}",
        my_page.addr() as usize,
        my_page.len()
    );
    let mut my_page = WritableRegion::from(my_page)?;
    assemble(&mut my_page[..]);
    for i in 0..8 {
        println!("{:2}: {:02X}", i, my_page[i]);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
const MAP_FAILED: *mut c_void = (!0usize) as *mut c_void;

struct MappedRegion {
    addr: *mut c_void,
    len: size_t,
}

impl MappedRegion {
    fn allocate(size: usize) -> Result<Self, &'static str> {
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

impl std::borrow::Borrow<[u8]> for MappedRegion {
    fn borrow(&self) -> &[u8] {
        &self[..]
    }
}

struct WritableRegion {
    region: MappedRegion,
}

impl WritableRegion {
    fn from(region: MappedRegion) -> Result<Self, &'static str> {
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
    I: std::slice::SliceIndex<[u8]>,
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
    I: std::slice::SliceIndex<[u8]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        unsafe {
            &mut std::slice::from_raw_parts_mut(self.region.addr as *mut u8, self.region.len)[index]
        }
    }
}

impl std::borrow::Borrow<[u8]> for WritableRegion {
    fn borrow(&self) -> &[u8] {
        &self.region[..]
    }
}

impl std::borrow::BorrowMut<[u8]> for WritableRegion {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self[..]
    }
}

impl std::ops::Drop for MappedRegion {
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

fn assemble(p: &mut [u8]) {
    // mov x0, #42
    // s op          hw imm16                Rd
    // 1 10 1|0010|1 00 0|0000|0000|0101|010 0|0000
    // D      2    8      0    0    5    4     0
    p[3] = 0xd2;
    p[2] = 0x80;
    p[1] = 0x05;
    p[0] = 0x40;

    // NB: x30 is the link register
    // ret x30
    //          opc   op2     op3     Rn     op4
    // 1101|011 0|100 1|1111 |0000|00 11|110 0|0000
    // D    6     9     F     0    3     C     0
    p[7] = 0xd6;
    p[6] = 0x5f;
    p[5] = 0x03;
    p[4] = 0xC0;
}
