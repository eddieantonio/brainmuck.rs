use std::borrow::{Borrow, BorrowMut};
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

use crate::ExecutableRegion;
use crate::MappedRegion;

pub struct WritableRegion {
    region: MappedRegion,
}

impl WritableRegion {
    pub fn from(region: MappedRegion) -> Result<Self, &'static str> {
        use libc::{PROT_READ, PROT_WRITE};

        unsafe {
            if libc::mprotect(region.addr_mut(), region.len(), PROT_READ | PROT_WRITE) < 0 {
                return Err("could not change protection");
            }
        }

        Ok(Self { region })
    }

    /// Consumes the region and returns an executable region. That means you can run it!
    pub fn to_executable(self) -> Result<ExecutableRegion, &'static str> {
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
