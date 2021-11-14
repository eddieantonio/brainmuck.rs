use mmap_jit::{as_function, WritableRegion};

use crate::ir::ControlFlowGraph;

type Program = fn(u64) -> u64;

#[derive(Clone, Copy)]
struct X(pub u8);
const SP: X = X(31);

#[derive(Clone, Copy)]
struct W(pub u8);

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
        self.emit(0)
    }

    // branch and line from register
    pub fn blr(&mut self, rd: X) {
        self.emit(0)
    }

    /// ret (return from subroutine)
    pub fn ret(&mut self) {
        self.emit(0xd65f03c0);
    }

    /// store pair of registers
    /// https://developer.arm.com/documentation/dui0801/h/A64-Data-Transfer-Instructions/STP
    pub fn stp_preindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        self.emit(0xA9BE7BFD);
    }

    pub fn ldp_postindex(&mut self, xt1: X, xt2: X, xn: X, imm: i16) {
        self.emit(0xA9BE7BFD);
    }

    /// store with immediate offset
    /// https://developer.arm.com/documentation/dui0802/a/CIHGJHED
    pub fn str_imm(&mut self, rt: X, rn: X, offset: i16) {
        self.emit(0);
    }

    pub fn ldr_imm(&mut self, rt: X, rn: X, offset: i16) {
        self.emit(0);
    }

    /// Subract (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-general-instructions/sub--immediate-?lang=en
    pub fn sub(&mut self, wd: W, wn: W, imm: u16) {
        self.emit(0)
    }

    /// Move (register)
    /// https://developer.arm.com/documentation/100069/0609/A64-General-Instructions/MOV--register-
    pub fn mov(&mut self, rd: X, op2: X) {
        self.emit(0);
    }

    /// Load Register Byte (immediate)
    pub fn ldrb(&mut self, wt: W, xn: X, pimm: u16) {
        self.emit(0);
    }

    /// Store Register Byte (immediate)
    /// https://developer.arm.com/documentation/100076/0100/a64-instruction-set-reference/a64-data-transfer-instructions/strb--immediate-?lang=en
    pub fn strb(&mut self, wt: W, xn: X, pimm: u16) {
        self.emit(0);
    }

    pub fn cbz(&mut self, rn: W, l: u16) {
        self.emit(0);
    }

    pub fn b(&mut self, l: u16) {
        self.emit(0);
    }
}

pub fn run(tac: &ControlFlowGraph) {
    let sample_program = [
        // mul x0, x0, x0
        0x00, 0x7c, 0x00, 0x9b, // ..
        // ret x30
        0xc0, 0x03, 0x5f, 0xd6,
    ];

    let mut machine_code = AArch64Assembly::new();

    // REGISTERS:
    //
    // x0                 - working byte
    //const VAL64: X = X(0);
    const VAL32: W = W(0);
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
    // x30                - stack pointer
    //
    // see: https://en.wikipedia.org/wiki/Calling_convention#ARM_(A64)
    //
    // STACK
    //
    // -0x40 [previous  fp]
    // -0x30 [previous  sp]
    // -0x20 [previous x19]
    // -0x10 [previous x20]
    //  0x00 [previous x21]
    //
    // fp <- sp
    // x19 <- x0

    // function intro
    //  1. setup stack
    //  stp	x29, x30, [sp, #-32]!
    machine_code.stp_preindex(X(29), X(30), SP, -32);
    //  mov	x29, sp
    machine_code.mov(X(29), SP);
    //  2. save registers
    //  stp	x20, x21, [sp, #-32]!
    machine_code.stp_preindex(GETCHAR, PUTCHAR, SP, -32);
    //  str	x19, [sp, #16]
    machine_code.str_imm(ADDR, SP, 16);

    // place the pointers somewhere safe
    machine_code.mov(PUTCHAR, X(1));
    machine_code.mov(GETCHAR, X(2));

    // generate instructions
    let instr = tac.blocks()[0].instructions()[0];
    use crate::ir::ThreeAddressInstruction::*;
    match instr {
        NoOp => (),
        ChangeAddr(x) => {
            // TODO: actually, should load the value if it's too big, then store it.
            machine_code.add(VAL32, VAL32, (x & 0xFFFF) as u16);
        }
        ChangeVal(x) => {
            // x0 <- *p
            machine_code.ldrb(VAL32, ADDR, 0);

            if (x as i8) >= 0 {
                // x0 <- x0 + x
                machine_code.add(VAL32, VAL32, x as u16);
            } else {
                // x0 <- x0 - x
                machine_code.sub(VAL32, VAL32, -(x as i8) as u16);
            }

            // *p = x0
            machine_code.strb(VAL32, ADDR, 0);
        }
        PutChar => {
            machine_code.ldrb(VAL32, ADDR, 0);
            machine_code.blr(PUTCHAR);
        }
        GetChar => {
            machine_code.blr(GETCHAR);
            machine_code.strb(VAL32, ADDR, 0);
        }
        BranchIfZero(_) => {
            // ldbr     x0, [x19]
            machine_code.ldrb(VAL32, ADDR, 0);
            // cbz    w0, L*
            machine_code.cbz(VAL32, 0);
        }
        BranchTo(_) => {
            // b    L*
            machine_code.b(0);
        }
        Terminate => {
            machine_code.ldr_imm(ADDR, SP, 16);
            machine_code.ldp_postindex(PUTCHAR, GETCHAR, SP, 16);
            machine_code.ldp_postindex(X(29), X(30), SP, 16);
            machine_code.ret();
        }
    }

    let mut mem = WritableRegion::allocate(sample_program.len()).unwrap();
    (&mut mem[0..sample_program.len()]).copy_from_slice(&sample_program);
    let code = mem.into_executable().unwrap();

    let program = unsafe { as_function!(code, Program) };
    let res = program(4);
    assert_eq!(16, res);
}
