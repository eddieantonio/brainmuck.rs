use mmap_jit::{as_function, WritableRegion};
use std::fmt;

use crate::ir::ControlFlowGraph;
use crate::ir::ThreeAddressInstruction;

type Program = fn(u64) -> u64;

#[derive(Clone, Copy)]
struct X(pub u8);
const SP: X = X(31);

impl fmt::Display for X {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 31 {
            write!(f, "sp")
        } else {
            write!(f, "x{}", self.0)
        }
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

impl AArch64Assembly {
    fn new() -> Self {
        AArch64Assembly { instr: Vec::new() }
    }

    fn emit(&mut self, instruction: u32) {
        let arr = instruction.to_le_bytes();
        self.instr.push(arr[0]);
        self.instr.push(arr[1]);
        self.instr.push(arr[2]);
        self.instr.push(arr[3]);
    }

    pub fn add(&mut self, wd: W, wn: W, imm: u16) {
        println!("add {}, {}, {}", wd, wn, imm);
        self.emit(0);
    }

    // branch and line from register
    pub fn blr(&mut self, rd: X) {
        println!("blr {}", rd);
        self.emit(0);
    }

    /// ret (return from subroutine)
    pub fn ret(&mut self) {
        println!("ret x30");
        self.emit(0xd65f03c0);
    }

    // also useful for addressing modes:
    // https://thinkingeek.com/2016/11/13/exploring-aarch64-assembler-chapter-5/

    /// store pair of registers
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_preindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        println!("stp {}, {}, [{}, #{}]!", xt1, xt2, xn, imm);
        self.emit(0xA9BE7BFD);
    }

    /// store pair of registers
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_offset(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        println!("stp {}, {}, [{}, #{}]", xt1, xt2, xn, imm);
        self.emit(0xA9BE7BFD);
    }

    pub fn ldp_postindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        println!("ldp {}, {}, [{}], #{}", xt1, xt2, xn, imm);
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        self.emit(0xA9BE7BFD);
    }

    pub fn ldp_offset(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        println!("ldp {}, {}, [{}, #{}]", xt1, xt2, xn, imm);
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        self.emit(0xA9BE7BFD);
    }

    /// store with immediate offset
    /// https://developer.arm.com/documentation/dui0802/a/CIHGJHED
    pub fn str_imm(&mut self, rt: X, rn: X, offset: i16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        println!("str {}, [{}, #{}]", rt, rn, offset);
        self.emit(0);
    }

    pub fn ldr_imm(&mut self, rt: X, rn: X, offset: i16) {
        println!("ldr {}, [{}, #{}]", rt, rn, offset);
        self.emit(0);
    }

    /// Subract (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-general-instructions/sub--immediate-?lang=en
    pub fn sub(&mut self, wd: W, wn: W, imm: u16) {
        println!("sub {}, {}, #{}", wd, wn, imm);
        self.emit(0);
    }

    /// Move (register)
    /// https://developer.arm.com/documentation/100069/0609/A64-General-Instructions/MOV--register-
    pub fn mov(&mut self, rd: X, op2: X) {
        println!("mov {}, {}", rd, op2);
        self.emit(0);
    }

    /// Load Register Byte (immediate)
    pub fn ldrb(&mut self, wt: W, xn: X, pimm: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        println!("ldrb {}, [{}, #{}]", wt, xn, pimm);
        self.emit(0);
    }

    /// Store Register Byte (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-data-transfer-instructions/strb--immediate-?lang=en
    pub fn strb(&mut self, wt: W, xn: X, pimm: u16) {
        // https://developer.arm.com/documentation/102374/0101/Loads-and-stores---addressing
        println!("strb {}, [{}, #{}]", wt, xn, pimm);
        self.emit(0);
    }

    pub fn cbz(&mut self, rn: W, l: u16) {
        println!("cbz {}, L{}", rn, l);
        self.emit(0);
    }

    pub fn b(&mut self, l: u16) {
        println!("b L{}", l);
        self.emit(0);
    }
}

// REGISTERS:
//
// x0                 - working byte
const VAL32: W = W(0);
// x19 (callee saved) - pointer to arena (during function)
const ADDR: X = X(19);
const ADDR32: W = W(19);
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
// see: https://en.wikipedia.org/wiki/Calling_convention#ARM_(A64)

pub fn run(tac: &ControlFlowGraph) {
    let sample_program = [
        // mul x0, x0, x0
        0x00, 0x7c, 0x00, 0x9b, // ..
        // ret x30
        0xc0, 0x03, 0x5f, 0xd6,
    ];

    let mut asm = AArch64Assembly::new();

    // godbolt.org is useful for generating instructions

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

    // function intro
    //  1. setup stack
    //  stp	x29, x30, [sp, #-48]!
    //  mov	x29, sp
    asm.stp_preindex(FP, LR, SP, -0x30);
    asm.mov(FP, SP);

    //  2. save registers
    //  stp x20, x21, [sp, 0x20]
    //  str	x19, [sp, 0x30]
    asm.stp_offset(PUTCHAR, GETCHAR, SP, 0x20);
    asm.str_imm(ADDR, SP, 0x30);

    // place the pointers somewhere safe
    asm.mov(ADDR, X(0));
    asm.mov(PUTCHAR, X(1));
    asm.mov(GETCHAR, X(2));

    // generate instructions
    for block in tac.blocks().iter() {
        for &instr in block.instructions().iter() {
            generate_instructions(&mut asm, instr);
        }
    }

    let mut mem = WritableRegion::allocate(sample_program.len()).unwrap();
    (&mut mem[0..sample_program.len()]).copy_from_slice(&sample_program);
    let code = mem.into_executable().unwrap();

    let program = unsafe { as_function!(code, Program) };
    let res = program(4);
    assert_eq!(16, res);
}

fn generate_instructions(asm: &mut AArch64Assembly, instr: ThreeAddressInstruction) {
    use ThreeAddressInstruction::*;
    match instr {
        NoOp => (),
        ChangeAddr(x) => {
            // TODO: actually, should load the value if it's too big, then store it.
            asm.add(ADDR32, ADDR32, (x & 0xFFFF) as u16);
        }
        ChangeVal(x) => {
            // x0 <- *p
            asm.ldrb(VAL32, ADDR, 0);

            if (x as i8) >= 0 {
                // x0 <- x0 + x
                asm.add(VAL32, VAL32, x as u16);
            } else {
                // x0 <- x0 - x
                asm.sub(VAL32, VAL32, -(x as i8) as u16);
            }

            // *p = x0
            asm.strb(VAL32, ADDR, 0);
        }
        PutChar => {
            asm.ldrb(VAL32, ADDR, 0);
            asm.blr(PUTCHAR);
        }
        GetChar => {
            asm.blr(GETCHAR);
            asm.strb(VAL32, ADDR, 0);
        }
        BranchIfZero(_) => {
            // ldbr     x0, [x19]
            asm.ldrb(VAL32, ADDR, 0);
            // cbz    w0, L*
            asm.cbz(VAL32, 0);
        }
        BranchTo(_) => {
            // b    L*
            asm.b(0);
        }
        Terminate => {
            // ldr x19, [sp, #0x30]
            // ldp x20, x21 [sp, #0x20]
            // ldp x29, x30 [sp], #0x20
            asm.ldr_imm(ADDR, SP, 0x30);
            asm.ldp_offset(GETCHAR, PUTCHAR, SP, 0x20);
            asm.ldp_postindex(FP, LR, SP, 0x30);
            asm.ret();
        }
    }
}
