use errno::errno;

use crate::MappedRegion;

/// An executable region of memory. Use [as_function!] to run code from here!
pub struct ExecutableRegion {
    region: MappedRegion,
}

impl ExecutableRegion {
    /// Consumes the [MappedRegion] and marks its memory as read-only and executable.
    pub fn from(region: MappedRegion) -> crate::Result<Self> {
        use libc::{PROT_EXEC, PROT_READ};

        unsafe {
            let addr = region.addr_mut() as *mut libc::c_void;
            if libc::mprotect(addr, region.len(), PROT_READ | PROT_EXEC) < 0 {
                return Err(errno().into());
            }
        }

        Ok(Self { region })
    }

    /// Returns the address of the mapped memory.
    ///
    /// Use [as_function!] to call this region of memory like a function.
    pub fn addr(&self) -> *const u8 {
        self.region.addr()
    }
}
