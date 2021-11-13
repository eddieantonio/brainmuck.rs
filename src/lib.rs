extern crate libc;

mod executable_region;
mod mapped_region;
mod writable_region;

pub use crate::executable_region::ExecutableRegion;
pub use crate::mapped_region::MappedRegion;
pub use crate::writable_region::WritableRegion;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    const MAPPING_SIZE: usize = 4096;

    #[test]
    fn mapping_gives_a_valid_address() -> Result<(), &'static str> {
        use libc::MAP_FAILED;

        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        assert_eq!(MAPPING_SIZE, region.len());
        assert_ne!(region.addr(), ptr::null());
        assert_ne!(region.addr(), MAP_FAILED);
        Ok(())
    }

    #[test]
    fn can_write_to_writable_mapping() -> Result<(), &'static str> {
        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        let mut p = WritableRegion::from(region)?;

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

        assert_eq!(0x40, p[0]);

        Ok(())
    }

    #[test]
    fn convert_writable_region_to_executable() -> Result<(), &'static str> {
        let region = MappedRegion::allocate(MAPPING_SIZE)?;
        let initial_addr = region.addr();
        let mut p = WritableRegion::from(region)?;

        // mov x0, #42
        p[3] = 0xd2;
        p[2] = 0x80;
        p[1] = 0x05;
        p[0] = 0x40;

        // ret x30
        p[7] = 0xd6;
        p[6] = 0x5f;
        p[5] = 0x03;
        p[4] = 0xC0;

        let exec = p.to_executable()?;
        assert_eq!(initial_addr, exec.addr());

        Ok(())
    }
}
