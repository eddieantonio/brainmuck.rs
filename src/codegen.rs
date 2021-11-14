use mmap_jit::{as_function, WritableRegion};
use std::fmt;

use crate::ir::BlockLabel;
use crate::ir::ControlFlowGraph;
use crate::ir::ThreeAddressInstruction;

// REGISTERS:
//
// x0                 - working byte
const VAL: W = W(0);
// x19 (callee saved) - pointer to arena (during function)
const ADDR: X = X(19);
// x20 (callee saved) - getchar (during function)
const GETCHAR: X = X(20);
// x21 (callee saved) - getchar (during function)
const PUTCHAR: X = X(21);
// x0  (argument)     - pointer to arena (as argument)
// x1  (argument)     - putchar
// x1  (argument)     - getchar
//
// x29                - frame pointer
const FP: X = X(29);
// x30                - link register
const LR: X = X(30);
//
// x31                - stack pointer or zero, depending on context
const SP: X = X(31);
// see: https://en.wikipedia.org/wiki/Calling_convention#ARM_(A64)
// also useful for addressing modes:
// https://thinkingeek.com/2016/11/13/exploring-aarch64-assembler-chapter-5/

macro_rules! asm {
    ($($fmt: expr),+) => {{
        print!("\t");
        println!($($fmt),+);
    }};
}

type Program = fn(u64) -> u64;

#[derive(Clone, Copy)]
struct X(pub u8);

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

#[derive(Clone, Copy)]
struct Imm(pub u8, pub i32);

impl BitPack for Imm {
    fn to_u32(self) -> u32 {
        self.1 as u32
    }
    fn expected_size(self) -> u8 {
        self.0
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

#[derive(Clone, Copy)]
struct W(pub u8);

impl fmt::Display for W {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "w{}", self.0)
    }
}

struct AArch64Assembly {
    instr: Vec<u8>,
}

pub fn run(cfg: &ControlFlowGraph) {
    let sample_program = [
        // mul x0, x0, x0
        0x00, 0x7c, 0x00, 0x9b, // ..
        // ret x30
        0xc0, 0x03, 0x5f, 0xd6,
    ];

    let mut code = CodeGenerator::new();
    code.generate(&cfg);

    let mut mem = WritableRegion::allocate(sample_program.len()).unwrap();
    (&mut mem[0..sample_program.len()]).copy_from_slice(&sample_program);
    let code = mem.into_executable().unwrap();

    let program = unsafe { as_function!(code, Program) };
    let res = program(4);
    assert_eq!(16, res);
}

impl AArch64Assembly {
    fn new() -> Self {
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

    // Data processing -- immediate

    // Branch, exception generation, and system instructions /////////

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

    // Loads and stores ////////////////////////////////////////////////

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

    // Data processing -- immediate /////////////////////////////////////////////////////

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

    // branch and line from register
    pub fn blr(&mut self, rd: X) {
        asm!("blr {}", rd);
    }

    /// ret (return from subroutine)
    pub fn ret(&mut self) {
        asm!("ret x30");
        self.emit(0xD65F03C0);
    }

    /// Move (register)
    /// https://developer.arm.com/documentation/100069/0609/A64-General-Instructions/MOV--register-
    pub fn mov(&mut self, rd: X, op2: X) {
        asm!("mov {}, {}", rd, op2);
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
}

struct CodeGenerator {
    asm: AArch64Assembly,
}

impl CodeGenerator {
    fn new() -> Self {
        CodeGenerator {
            asm: AArch64Assembly::new(),
        }
    }

    pub fn generate(&mut self, cfg: &ControlFlowGraph) {
        asm!(".globl _bf_program");
        asm!(".p2align 2");
        println!("_bf_program:");

        // function intro
        self.setup_stack();
        self.save_registers();

        self.generate_code(&cfg);
        assert!(matches!(
            cfg.last_instruction(),
            Some(ThreeAddressInstruction::Terminate)
        ));
    }

    // STACK
    //
    // sp + 0x30 [previous x19]
    // sp + 0x28 [ ..unused   ]
    // sp + 0x20 [previous x20]
    // sp + 0x18 [previous x21]
    // sp + 0x10 [previous  fp]
    // sp + 0x08 [previous  lr]
    // fp -> 0
    //
    // fp <- sp
    // x19 <- x0
    // x20 <- x1
    // x21 <- x2

    fn setup_stack(&mut self) {
        //  stp	x29, x30, [sp, #-48]!
        //  mov	x29, sp
        self.asm.stp_preindex(FP, LR, SP, -0x30);
        self.asm.mov(FP, SP);
    }

    fn save_registers(&mut self) {
        //  stp x20, x21, [sp, 0x20]
        //  str	x19, [sp, 0x30]
        self.asm.stp_offset(PUTCHAR, GETCHAR, SP, 0x20);
        self.asm.str_imm(ADDR, SP, 0x30);

        // mov x19, x0
        // mov x20, x1
        // mov x21, x2
        self.asm.mov(ADDR, X(0));
        self.asm.mov(PUTCHAR, X(1));
        self.asm.mov(GETCHAR, X(2));
    }

    fn restore_stack_and_registers_and_return(&mut self) {
        // ldr x19, [sp, #0x30]
        // ldp x20, x21 [sp, #0x20]
        // ldp x29, x30 [sp], #0x20
        self.asm.ldr_imm(ADDR, SP, 0x30);
        self.asm.ldp_offset(GETCHAR, PUTCHAR, SP, 0x20);
        self.asm.ldp_postindex(FP, LR, SP, 0x30);
        self.asm.ret();
    }

    fn generate_code(&mut self, cfg: &ControlFlowGraph) {
        for block in cfg.blocks().iter() {
            let BlockLabel(l) = block.label();
            println!("L{}:", l);
            for &instr in block.instructions().iter() {
                self.generate_instructions(instr);
            }
        }
    }

    fn generate_instructions(&mut self, instr: ThreeAddressInstruction) {
        use ThreeAddressInstruction::*;
        match instr {
            NoOp => (),
            ChangeAddr(x) => {
                // FIXME: this is wrong; it should be using 64-bit add/sub
                if x == 0 {
                    return;
                }
                if x >= 0 {
                    self.asm.add64(ADDR, ADDR, x as u16);
                } else {
                    self.asm.sub64(ADDR, ADDR, (-x) as u16);
                }
            }
            ChangeVal(x) => {
                // x0 <- *p
                self.asm.ldrb(VAL, ADDR, 0);

                if (x as i8) >= 0 {
                    // x0 <- x0 + x
                    self.asm.add(VAL, VAL, x as u16);
                } else {
                    // x0 <- x0 - x
                    self.asm.sub(VAL, VAL, -(x as i8) as u16);
                }

                // *p = x0
                self.asm.strb(VAL, ADDR, 0);
            }
            PutChar => {
                self.asm.ldrb(VAL, ADDR, 0);
                self.asm.blr(PUTCHAR);
            }
            GetChar => {
                self.asm.blr(GETCHAR);
                self.asm.strb(VAL, ADDR, 0);
            }
            BranchIfZero(BlockLabel(l)) => {
                // ldbr     x0, [x19]
                self.asm.ldrb(VAL, ADDR, 0);
                // cbz    w0, L*
                self.asm.cbz(VAL, l as i32);
            }
            BranchTo(BlockLabel(l)) => {
                // b    L*
                self.asm.b(l as i32);
            }
            Terminate => {
                self.restore_stack_and_registers_and_return();
            }
        }
    }
}
