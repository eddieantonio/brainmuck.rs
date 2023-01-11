//! Assembler for ARM AArch64

use std::collections::HashMap;
use std::fmt;

// This is used for debug prints, but I deleted them :3
macro_rules! asm {
    ($($fmt: expr),+) => {{}};
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

/// An unsigned immediate value in the instruction.
#[derive(Clone, Copy)]
pub struct Umm(pub u8, pub u32);

/// A branch label in the assembly
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Label(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct WordOffset(i32);

#[derive(Clone, Copy)]
enum IncompleteInstruction {
    Cbz,
    B,
}

/// Generates ARM AArch64 machine code.
pub struct AArch64Assembly {
    instr: Vec<u8>,
    // Maps labels to the offset in the instruction vector
    label_targets: HashMap<Label, WordOffset>,
    //
    unresolved_branch_targets: Vec<(WordOffset, IncompleteInstruction, Label)>,
}

impl AArch64Assembly {
    // I'm using bit groupings used in the ARM binary encoding spec, which are NOT 4 bit aligned!
    #![allow(clippy::unusual_byte_groupings)]

    pub fn new() -> Self {
        AArch64Assembly {
            instr: Vec::new(),
            label_targets: HashMap::new(),
            unresolved_branch_targets: Vec::new(),
        }
    }

    /// Call this before the first instruction of the desired label
    pub fn set_label_target(&mut self, label: Label) {
        let offset = WordOffset::from_byte_offset(self.instr.len());
        self.label_targets.insert(label, offset);
    }

    pub fn patch_branch_targets(&mut self) {
        let patch_list = self.unresolved_branch_targets.clone();
        for (source, instr, label) in patch_list {
            let target = self
                .label_targets
                .get(&label)
                .expect("should have seen label");
            let incomplete = self.get_instruction(source);

            let offset = *target - source;

            let missing_bits = match instr {
                IncompleteInstruction::Cbz => Self::patch_cbz(offset),
                IncompleteInstruction::B => Self::patch_b(offset),
            };

            let complete = incomplete | missing_bits;

            self.set_instruction(source, complete);
        }

        self.unresolved_branch_targets.clear();
    }

    fn get_instruction(&self, offset: WordOffset) -> u32 {
        let n_bytes = offset.to_usize();

        let mut word = [0u8; 4];
        word.copy_from_slice(&self.instr[n_bytes..(n_bytes + 4)]);

        u32::from_le_bytes(word)
    }

    fn set_instruction(&mut self, offset: WordOffset, instr: u32) {
        let n_bytes = offset.to_usize();
        let bytes = instr.to_le_bytes();
        self.instr[n_bytes..(n_bytes + 4)].copy_from_slice(&bytes);
    }

    /// Returns machine code.
    /// Panics if there are unresolved branch targets.
    pub fn machine_code(&self) -> &[u8] {
        let incomplete = self.unresolved_branch_targets.len();
        if incomplete > 0 {
            panic!(
                "tried to generate binary, but there are still {} unresolved branch targets!",
                incomplete
            );
        }

        &self.instr[..]
    }

    // Instructions
    //
    // The following instructions are in the order given by
    // Chapter C3 - A64 Instruction Set Encoding

    // Branch, exception generation, and system instructions //////////////////////////////////////

    /// Compare register and Branch if Zero
    pub fn cbz(&mut self, rt: W, label: Label) {
        use IncompleteInstruction::Cbz;
        asm!("cbz {}, {}", rt, label);
        //          sf ______ op              imm19    rt
        //                      23                5 4   0
        let base = 0b0_011010_0_0000000000000000000_00000;
        self.emit_incomplete_branch(label, Cbz, base | rt.at(0..=4));
    }

    fn patch_cbz(offset: WordOffset) -> u32 {
        let WordOffset(imm) = offset;
        Imm(19, imm).at(5..=23)
    }

    /// Unconditional branch
    pub fn b(&mut self, label: Label) {
        use IncompleteInstruction::B;
        asm!("b {}", label);
        //          op                            imm26
        let base = 0b0_00101_00000000000000000000000000;
        self.emit_incomplete_branch(label, B, base);
    }

    fn patch_b(offset: WordOffset) -> u32 {
        let WordOffset(imm) = offset;
        Imm(26, imm).at(0..=25)
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

    // Load/store register (unsigned immediate)

    /// Store Register Byte (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-data-transfer-instructions/strb--immediate-?lang=en
    pub fn strb(&mut self, wt: W, xn: X, offset: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("strb {}, [{}, #{}]", wt, xn, offset);
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (offset >> 3) as i32;
        //         size     V   opc        imm12    rn    rt
        let base = 0b00_111_0_01_00_000000000000_00000_00000;
        self.emit(base | wt.at(0..=4) | xn.at(5..=9) | Imm(12, dword_aligned_offset).at(10..=21));
    }

    /// Load Register Byte (immediate)
    pub fn ldrb(&mut self, wt: W, xn: X, offset: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("ldrb {}, [{}, #{}]", wt, xn, offset);
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (offset >> 3) as i32;
        //         size     V   opc        imm12    rn    rt
        let base = 0b00_111_0_01_01_000000000000_00000_00000;
        self.emit(base | wt.at(0..=4) | xn.at(5..=9) | Imm(12, dword_aligned_offset).at(10..=21));
    }

    /// Store dword register with immediate offset
    /// https://developer.arm.com/documentation/dui0802/a/CIHGJHED
    pub fn str_imm(&mut self, rt: X, rn: X, offset: u16) {
        asm!("str {}, [{}, #{}]", rt, rn, offset);
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (offset >> 3) as i32;
        //         size     V   opc        imm12    rn    rt
        let base = 0b11_111_0_01_00_000000000000_00000_00000;
        self.emit(base | rt.at(0..=4) | rn.at(5..=9) | Imm(12, dword_aligned_offset).at(10..=21));
    }

    /// Load dword register with unsigned immediate offset
    pub fn ldr_imm(&mut self, rt: X, rn: X, offset: i16) {
        asm!("ldr {}, [{}, #{}]", rt, rn, offset);
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (offset >> 3) as i32;
        //         size     V   opc        imm12    rn    rt
        let base = 0b11_111_0_01_01_000000000000_00000_00000;
        self.emit(base | rt.at(0..=4) | rn.at(5..=9) | Imm(12, dword_aligned_offset).at(10..=21));
    }

    // Load/store register pair (unsigned offset)

    /// Store pair of registers (unsigned offset)
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_offset(&mut self, rt: X, rt2: X, rn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("stp {}, {}, [{}, #{}]", rt, rt2, rn, imm);
        //          opc     V     L    imm7   rt2    rn    rt
        let base = 0b10_101_0_010_0_0000000_00000_00000_00000;
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (imm >> 3) as u32;
        self.emit(
            base | rt.at(0..=4)
                | rn.at(5..=9)
                | rt2.at(10..=14)
                | Umm(7, dword_aligned_offset).at(15..=21),
        );
    }

    /// Load pair of registers (unsigned offset)
    pub fn ldp_offset(&mut self, rt: X, rt2: X, rn: X, imm: i16) {
        asm!("ldp {}, {}, [{}, #{}]", rt, rt2, rn, imm);
        //          opc     V     L    imm7   rt2    rn    rt
        let base = 0b10_101_0_010_1_0000000_00000_00000_00000;
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (imm >> 3) as u32;
        self.emit(
            base | rt.at(0..=4)
                | rn.at(5..=9)
                | rt2.at(10..=14)
                | Umm(7, dword_aligned_offset).at(15..=21),
        );
    }

    // Load/store register pair (post-index)
    // post-index means that the dword-aligned offset will be added
    // AFTER indexing (e.g., like *p++ in C).

    /// Load pair of registers (post-index)
    pub fn ldp_postindex(&mut self, rt1: X, rt2: X, rn: X, imm: i16) {
        asm!("ldp {}, {}, [{}], #{}", rt1, rt2, rn, imm);
        //          opc     V     L    imm7   rt2    rn    rt
        let base = 0b10_101_0_001_1_0000000_00000_00000_00000;
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (imm >> 3) as i32;
        self.emit(
            base | rt1.at(0..=4)
                | rn.at(5..=9)
                | rt2.at(10..=14)
                | Imm(7, dword_aligned_offset).at(15..=21),
        );
    }

    // Load/store register pair (pre-indexed)
    // The dword-aligned offset index is added

    /// Store Pair of registers (pre-indexed)
    pub fn stp_preindex(&mut self, rt1: X, rt2: X, rn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        asm!("stp {}, {}, [{}, #{}]!", rt1, rt2, rn, imm);
        //          opc     V     L    imm7   rt2    rn    rt
        let base = 0b10_101_0_011_0_0000000_00000_00000_00000;
        // Offset is described in bytes, but must be 8-byte aligned (lower 3 bits are implied 0)
        let dword_aligned_offset = (imm >> 3) as i32;
        self.emit(
            base | rt1.at(0..=4)
                | rn.at(5..=9)
                | rt2.at(10..=14)
                | Imm(7, dword_aligned_offset).at(15..=21),
        );
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

    /// Move register (shh! this is secretly ORR)
    pub fn mov(&mut self, rd: X, rm: X) {
        asm!("mov {0}, {1} ; orr {0}, x31, {1}", rd, rm);
        //          sf op       << N    rm   imm6    rn    rd
        let base = 0b1_01_01010_00_0_00000_000000_00000_00000;
        self.emit(base | rm.at(16..=20) | X(31).at(5..=9) | rd.at(0..=4));
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

    // Private methods ////////////////////////////////////////////////////////////////////////////

    fn emit(&mut self, instruction: u32) {
        let arr = instruction.to_le_bytes();
        self.instr.extend_from_slice(&arr);
    }

    fn emit_incomplete_branch(
        &mut self,
        label: Label,
        which: IncompleteInstruction,
        partial_instruction: u32,
    ) {
        // must calculate offset before emitting the instruction
        let offset = WordOffset::from_byte_offset(self.instr.len());
        self.emit(partial_instruction);
        self.unresolved_branch_targets.push((offset, which, label));
    }
}

impl WordOffset {
    /// Return a word offset from a byte offset
    pub fn from_byte_offset(n_bytes: usize) -> WordOffset {
        assert_eq!(n_bytes & 0b11, 0, "expected a 4-byte aligned offset");
        WordOffset((n_bytes / 4) as i32)
    }

    pub fn to_usize(self) -> usize {
        let WordOffset(words) = self;
        assert!(words >= 0);

        words as usize * 4
    }
}

impl std::ops::Sub for WordOffset {
    type Output = Self;
    fn sub(self, other: WordOffset) -> Self::Output {
        let WordOffset(a) = self;
        let WordOffset(b) = other;

        WordOffset(a - b)
    }
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
        let raw_bits = self.1 as u32;
        // Keep only the bits that contribute to the immediate value:
        let mask = 2u32.pow(self.expected_size() as u32) - 1;
        mask & raw_bits
    }
    fn expected_size(self) -> u8 {
        self.0
    }
}

impl BitPack for Umm {
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

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "L{}", self.0)
    }
}
