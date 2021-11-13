use libc::c_void;

use crate::MappedRegion;

pub struct ExecutableRegion {
    region: MappedRegion,
}

impl ExecutableRegion {
    pub fn from(region: MappedRegion) -> Result<Self, &'static str> {
        use libc::{PROT_EXEC, PROT_READ};

        unsafe {
            if libc::mprotect(region.addr_mut(), region.len(), PROT_READ | PROT_EXEC) < 0 {
                return Err("could not change protection");
            }
        }

        Ok(Self { region })
    }

    pub fn addr(&self) -> *const c_void {
        self.region.addr()
    }
}
