use mmap_jit::{as_function, WritableRegion};

use crate::asm::aarch64::{AArch64Assembly, W, X};
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
