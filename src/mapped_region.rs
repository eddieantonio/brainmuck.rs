use std::borrow::Borrow;
use std::ops::{Drop, Index};
use std::ptr;

use errno::errno;
use libc::{c_void, size_t};

use crate::WritableRegion;

#[cfg(target_os = "macos")]
const MAP_FAILED: *mut c_void = (!0usize) as *mut c_void;

/// A region of memory mapped by `mmap(2)`.
///
/// The `munmap(2)` is automatically called when the value is dropped.
pub struct MappedRegion {
    addr: *mut c_void,
    len: size_t,
}

impl MappedRegion {
    /// Allocate a region of the given size (in bytes).
    pub fn allocate(size: usize) -> crate::Result<Self> {
        use libc::{MAP_ANON, MAP_JIT, MAP_PRIVATE};
        let memory;
        unsafe {
            memory = libc::mmap(
                ptr::null_mut(),
                size,
                0,
                MAP_PRIVATE | MAP_ANON | MAP_JIT,
                -1,
                0,
            );
        }

        if memory == MAP_FAILED {
            return Err(errno().into());
        }

        Ok(MappedRegion {
            addr: memory,
            len: size,
        })
    }

    /// Returns a pointer to mapped memory.
    pub fn addr(&self) -> *const c_void {
        self.addr
    }

    /// Returns a mutable pointer to this region.
    ///
    /// Note: to write to this memory, first you must convert into a WritableRegion.
    pub fn addr_mut(&self) -> *mut c_void {
        self.addr
    }

    /// Return the length of region.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Consumes the region and returns a writable region.
    pub fn into_writable(self) -> crate::Result<WritableRegion> {
        WritableRegion::from(self)
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        unsafe {
            // TODO: check return value
            libc::munmap(self.addr, self.len);
        }
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
