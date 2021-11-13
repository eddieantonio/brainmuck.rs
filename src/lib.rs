//! Map some memory for writing and executing.
//!
//! This crate is a wrapper around `mmap(2)`, `mprotect(2)`, and `munmap(2)` calls that uses Rust's
//! type system to enforce what you can and can't do with a dynamically mapped region of memory.
//! The intent is to allocate memory in order to inject machine code into the running executable
//! and run it. This allows you to create, among other things, a JIT compiler.
//!
//! Here is the general workflow:
//!
//! ```
//! extern crate mmap_jit;
//!
//! use mmap_jit::{MappedRegion, as_function};
//!
//! // Allocate some amount of memory.
//! let mem = MappedRegion::allocate(4096).unwrap();
//!
//! // Make it writable.
//! let mut mem = mem.into_writable().unwrap();
//!
//! // Write to your memory!
//! mem[0] = 0xC3;
//!
//! // Make it executable.
//! let code = mem.into_executable().unwrap();
//!
//! // Congrats, now you have a function!
//! let f = unsafe { as_function!(code, fn() -> ()) };
//! ```

extern crate errno;
extern crate libc;

mod error;
mod executable_region;
mod mapped_region;
mod writable_region;

pub use crate::executable_region::ExecutableRegion;
pub use crate::mapped_region::MappedRegion;
pub use crate::writable_region::WritableRegion;

pub use crate::error::{MappingError, Result};

/// Cast an [ExecutableRegion] to a function pointer of your choosing.
///
/// Usage:
/// ```ignore
/// let e: ExecutableRegion = generate_code();
/// let f = unsafe { as_function!(e, fn(u64) -> u64) };
/// println!("use f(2) = {}", f(2));
/// ```
#[macro_export]
macro_rules! as_function {
    ($region: expr, $fn_type: ty) => {
        std::mem::transmute::<*const u8, $fn_type>($region.addr())
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    const MAPPING_SIZE: usize = 4096;

    #[test]
    fn mapping_gives_a_valid_address() -> Result<()> {
        use libc::{c_void, MAP_FAILED};

        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        assert_eq!(MAPPING_SIZE, region.len());
        assert_ne!(region.addr() as *const c_void, ptr::null());
        assert_ne!(region.addr() as *const c_void, MAP_FAILED);
        Ok(())
    }

    #[test]
    fn can_write_to_writable_mapping() -> Result<()> {
        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        let mut p = WritableRegion::from(region)?;
        write_return_42_function(&mut p);

        assert_eq!(0x40, p[0]);

        Ok(())
    }

    #[test]
    fn convert_writable_region_to_executable() -> Result<()> {
        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        let initial_addr = region.addr();

        let mut p = WritableRegion::from(region)?;
        write_square_function(&mut p);

        let exec = p.into_executable()?;
        assert_eq!(initial_addr, exec.addr());

        let function = unsafe { as_function!(exec, fn(u64) -> u64) };
        let res = function(4);
        assert_eq!(16, res);

        Ok(())
    }

    fn write_square_function(p: &mut WritableRegion) {
        let instr = 0x9b007c00u32;
        p[0..4].copy_from_slice(&instr.to_ne_bytes());

        let instr = 0xd65f03c0u32;
        p[4..8].copy_from_slice(&instr.to_ne_bytes());
    }

    /// Writes (little-endian) AArch64 machine code to the writable region.
    /// The program returns 42.
    fn write_return_42_function(p: &mut WritableRegion) {
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
        // 1101|011 0|010 1|1111 |0000|00 11|110 0|0000
        // D    6     5     F     0    3     C     0
        p[7] = 0xd6;
        p[6] = 0x5f;
        p[5] = 0x03;
        p[4] = 0xC0;
    }

    #[test]
    fn should_error_if_mapping_entire_address_space() {
        use errno::Errno;

        match MappedRegion::allocate(usize::MAX) {
            Ok(_) => {
                panic!("that should not have worked...");
            }
            Err(MappingError::Internal(Errno(c))) => {
                assert!(c > 0, "expected an error value, such as EINVAL");
            }
        }
    }
}
