//! Map some memory for writing and executing.
//!
//! This crate is a wrapper around `mmap(2)`, `mprotect(2)`, and `munmap(2)` calls that uses Rust's
//! type system to enforce what you can and can't do with a dynamically mapped region of memory.
//! The intent is to allocate memory in order to inject machine code into the running executable
//! and run it. This allows you to create, among other things, a JIT compiler.
//!
//! # Examples
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

pub mod examples;

pub use crate::executable_region::ExecutableRegion;
pub use crate::mapped_region::MappedRegion;
pub use crate::writable_region::WritableRegion;

pub use crate::error::{MappingError, Result};

/// Cast an [ExecutableRegion] to a function pointer of your choosing.
///
/// # Examples
///
/// ```
/// use mmap_jit::{self, as_function, ExecutableRegion};
/// let e: ExecutableRegion = mmap_jit::examples::generate_square_program();
/// let f = unsafe { as_function!(e, fn(u64) -> u64) };
/// assert_eq!(16, f(4));
/// ```
///
/// # Saftey
///
/// This is incredibly `unsafe`! You are responsible for writing a program that obeys the target
/// platform's ABI and additionally, does not invalidate any of Rust's assumptions about the state
/// of memory. The power is in your hands.
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
        let memory = &mut p[..];

        // Just write some bits to it
        memory[0] = 42;
        assert_eq!(42, p[0]);

        Ok(())
    }

    #[test]
    fn convert_writable_region_to_executable() -> Result<()> {
        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        let initial_addr = region.addr();

        let mut p = WritableRegion::from(region)?;
        examples::write_square_function(&mut p[..]);

        let exec = p.into_executable()?;
        assert_eq!(initial_addr, exec.addr());

        let function = unsafe { as_function!(exec, fn(u64) -> u64) };
        let res = function(4);
        assert_eq!(16, res);

        Ok(())
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
