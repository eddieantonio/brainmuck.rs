use std::borrow::{Borrow, BorrowMut};
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use errno::errno;

use crate::ExecutableRegion;
use crate::MappedRegion;

/// A memory-mapped region that can be written to.
///
/// Indexing and borrowing from the WritableRegion returns `[u8]`.
///
/// ```
/// use mmap_jit::WritableRegion;
///
/// let mut w = WritableRegion::allocate(1024).unwrap();
/// w[0] = 42;
/// assert_eq!(w[0], 42);
///
/// // Write multiple values at once:
/// let num: u32 = 0xDEADBEEF;
/// w[0..4].copy_from_slice(&num.to_ne_bytes());
///
/// let mut arr = [0u8;4];
/// // Borrow:
/// arr.copy_from_slice(&w[0..4]);
/// assert_eq!(0xDEADBEEF, u32::from_ne_bytes(arr));
/// ```
pub struct WritableRegion {
    region: MappedRegion,
}

impl WritableRegion {
    /// Consumes the existing [MappedRegion] and makes its memory writable.
    pub fn from(region: MappedRegion) -> crate::Result<Self> {
        use libc::{PROT_READ, PROT_WRITE};

        unsafe {
            if libc::mprotect(region.addr_mut(), region.len(), PROT_READ | PROT_WRITE) < 0 {
                return Err(errno().into());
            }
        }

        Ok(Self { region })
    }

    /// Convenience function to allocate a region and mark it writable in one go.
    pub fn allocate(size: usize) -> crate::Result<Self> {
        let region = MappedRegion::allocate(size)?;
        WritableRegion::from(region)
    }

    /// Consumes the region and returns an read-only, [ExecutableRegion].
    pub fn into_executable(self) -> crate::Result<ExecutableRegion> {
        ExecutableRegion::from(self.region)
    }
}

impl<I> Index<I> for WritableRegion
where
    I: SliceIndex<[u8]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        unsafe {
            &std::slice::from_raw_parts(self.region.addr() as *const u8, self.region.len())[index]
        }
    }
}

impl<I> IndexMut<I> for WritableRegion
where
    I: SliceIndex<[u8]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        unsafe {
            &mut std::slice::from_raw_parts_mut(
                self.region.addr_mut() as *mut u8,
                self.region.len(),
            )[index]
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
