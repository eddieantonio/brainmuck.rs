use libc::c_void;

use crate::MappedRegion;

/// An executable region of memory. Use [as_function!] to run code from here!
pub struct ExecutableRegion {
    region: MappedRegion,
}

impl ExecutableRegion {
    /// Consumes the [MappedRegion] and marks its memory as read-only and executable.
    pub fn from(region: MappedRegion) -> Result<Self, &'static str> {
        use libc::{PROT_EXEC, PROT_READ};

        unsafe {
            if libc::mprotect(region.addr_mut(), region.len(), PROT_READ | PROT_EXEC) < 0 {
                return Err("could not change protection");
            }
        }

        Ok(Self { region })
    }

    /// Returns the address of the mapped memory.
    ///
    /// Use [as_function!] to call this region of memory like a function.
    pub fn addr(&self) -> *const c_void {
        self.region.addr()
    }
}
