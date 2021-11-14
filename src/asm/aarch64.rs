//! Assembler for ARM AArch64

use std::fmt;

macro_rules! asm {
    ($($fmt: expr),+) => {{
        print!("\t");
        println!($($fmt),+);
    }};
}

/// Reference to 64-bit register
#[derive(Clone, Copy)]
pub struct X(pub u8);

/// Reference to low 32-bits of the register
#[derive(Clone, Copy)]
pub struct W(pub u8);

/// An immediate value in the instruction.
#[derive(Clone, Copy)]
pub struct Imm(pub u8, pub i32);

/// Generates ARM AArch64 machine code.
pub struct AArch64Assembly {
    instr: Vec<u8>,
}

impl AArch64Assembly {
    pub fn new() -> Self {
        AArch64Assembly { instr: Vec::new() }
    }

    fn emit(&mut self, instruction: u32) {
        let arr = instruction.to_le_bytes();
        self.instr.extend_from_slice(&arr);
        println!("\t{:04X}", instruction);
    }

    // Instructions
    //
    // The following instructions are in the order given by
    // Chapter C3 - A64 Instruction Set Encoding

    // Branch, exception generation, and system instructions //////////////////////////////////////

    /// Compare register and Branch if Zero
    pub fn cbz(&mut self, rt: W, l: i32) {
        asm!("cbz {}, L{}", rt, l);

        //          sf ______ op              imm19    rt
        //                      23                5 4   0
        let base = 0b0_011010_0_0000000000000000000_00000;
        self.emit(base | Imm(19, l).at(5..=23) | rt.at(0..=4));
    }

    /// Unconditional branch
    pub fn b(&mut self, l: i32) {
        asm!("b L{}", l);
        //          op                            imm26
        let base = 0b0_00101_00000000000000000000000000;
        self.emit(base | Imm(26, l).at(0..=25));
    }

    /// Branch and Link to Register
    pub fn blr(&mut self, rn: X) {
        asm!("blr {}", rn);
        //                   opc    op2    op3    rn   op4;
        let base = 0b1101011_0001_11111_000000_00000_00000;
        self.emit(base | rn.at(5..=9));
    }

    /// ret (return from subroutine)
    pub fn ret(&mut self) {
        asm!("ret x30");
        let base = 0b1101011_0010_11111_000000_00000_00000;
        self.emit(base | X(30).at(5..=9));
    }

    // Load and stores ////////////////////////////////////////////////////////////////////////////

    /// store pair of registers
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_preindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("stp {}, {}, [{}, #{}]!", xt1, xt2, xn, imm);
    }

    /// store pair of registers
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_offset(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("stp {}, {}, [{}, #{}]", xt1, xt2, xn, imm);
    }

    pub fn ldp_postindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        asm!("ldp {}, {}, [{}], #{}", xt1, xt2, xn, imm);
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
    }

    pub fn ldp_offset(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        asm!("ldp {}, {}, [{}, #{}]", xt1, xt2, xn, imm);
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
    }

    /// store with immediate offset
    /// https://developer.arm.com/documentation/dui0802/a/CIHGJHED
    pub fn str_imm(&mut self, rt: X, rn: X, offset: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("str {}, [{}, #{}]", rt, rn, offset);
    }

    pub fn ldr_imm(&mut self, rt: X, rn: X, offset: i16) {
        asm!("ldr {}, [{}, #{}]", rt, rn, offset);
    }

    /// Load Register Byte (immediate)
    pub fn ldrb(&mut self, wt: W, xn: X, pimm: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("ldrb {}, [{}, #{}]", wt, xn, pimm);
    }

    /// Store Register Byte (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-data-transfer-instructions/strb--immediate-?lang=en
    pub fn strb(&mut self, wt: W, xn: X, pimm: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("strb {}, [{}, #{}]", wt, xn, pimm);
    }

    // Data processing -- immediate ///////////////////////////////////////////////////////////////

    pub fn add(&mut self, wd: W, wn: W, imm: u16) {
        asm!("add {}, {}, {}", wd, wn, imm);
        //          sfop S       <<        imm12 Rn    Rd
        let base = 0b0_0_0_10001_00_000000000000_00000_00000;
        self.emit(base | Imm(12, imm as i32).at(10..=21) | wn.at(5..=9) | wd.at(0..=4));
    }

    pub fn add64(&mut self, xd: X, xn: X, imm: u16) {
        asm!("add {}, {}, {}", xd, xn, imm);
        //          sfop S       <<        imm12 Rn    Rd
        let base = 0b1_0_0_10001_00_000000000000_00000_00000;
        self.emit(base | Imm(12, imm as i32).at(10..=21) | xn.at(5..=9) | xd.at(0..=4));
    }

    /// Move (register) -- shh! Secretly this is an add!
    /// https://developer.arm.com/documentation/100069/0609/A64-General-Instructions/MOV--register-
    pub fn mov(&mut self, rd: X, rn: X) {
        asm!("mov {}, {}", rd, rn);
        //
        //          sf op
        //          sfop S       <<        imm12 Rn    Rd
        let base = 0b1_0_0_10001_00_000000000000_11111_00000;
        self.emit(base | rn.at(5..=9) | rd.at(0..=4));
    }

    /// Subract (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-general-instructions/sub--immediate-?lang=en
    pub fn sub(&mut self, wd: W, wn: W, imm: u16) {
        asm!("sub {}, {}, #{}", wd, wn, imm);
        //          sfop S       <<        imm12 Rn    Rd
        let base = 0b0_1_0_10001_00_000000000000_00000_00000;
        self.emit(base | Imm(12, imm as i32).at(10..=21) | wn.at(5..=9) | wd.at(0..=4));
    }

    pub fn sub64(&mut self, xd: X, xn: X, imm: u16) {
        asm!("sub {}, {}, #{}", xd, xn, imm);
        //          sfop S       <<        imm12 Rn    Rd
        let base = 0b1_1_0_10001_00_000000000000_00000_00000;
        self.emit(base | Imm(12, imm as i32).at(10..=21) | xn.at(5..=9) | xd.at(0..=4));
    }

    // Data processing -- register //////////////////////////////////////////////////////
}

/////////////////////////////////// Traits and implementations ////////////////////////////////////

trait BitPack: Copy {
    fn to_u32(self) -> u32;
    fn expected_size(self) -> u8;
    fn at(self, bits: std::ops::RangeInclusive<u8>) -> u32 {
        assert_eq!(
            1 + bits.end() - bits.start(),
            self.expected_size(),
            "unexpected size of bits for type"
        );
        self.to_u32() << bits.start()
    }
}

impl BitPack for X {
    fn to_u32(self) -> u32 {
        self.0 as u32
    }
    fn expected_size(self) -> u8 {
        5
    }
}

impl BitPack for W {
    fn to_u32(self) -> u32 {
        self.0 as u32
    }
    fn expected_size(self) -> u8 {
        5
    }
}

impl BitPack for Imm {
    fn to_u32(self) -> u32 {
        self.1 as u32
    }
    fn expected_size(self) -> u8 {
        self.0
    }
}

impl fmt::Display for W {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "w{}", self.0)
    }
}

impl fmt::Display for X {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 31 {
            write!(f, "sp")
        } else {
            write!(f, "x{}", self.0)
        }
    }
}
