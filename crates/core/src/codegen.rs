//! Generates machine code for a given program.

use crate::asm::aarch64::{AArch64Assembly, Label, W, X};
use crate::ir::BlockLabel;
use crate::ir::ControlFlowGraph;
use crate::ir::ThreeAddressInstruction;

// REGISTERS:
//
// x0                 - working byte
const VAL: W = W(0);
// x19 (callee saved) - current pointer on the "tape" (during function)
const ADDR: X = X(19);
// x20 (callee saved) - getchar (during function)
const GETCHAR: X = X(20);
// x21 (callee saved) - getchar (during function)
const PUTCHAR: X = X(21);
// x0  (argument)     - pointer to universe (as argument)
// x1  (argument)     - putchar (as argument)
// x1  (argument)     - getchar (as argument)
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

/// Takes three-address code and compiles it an executable.
pub struct CodeGenerator {
    asm: AArch64Assembly,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            asm: AArch64Assembly::new(),
        }
    }

    pub fn compile(&mut self, cfg: &ControlFlowGraph) -> &[u8] {
        self.setup_stack_and_save_registers();

        self.generate_code(cfg);
        assert!(
            matches!(
                cfg.last_instruction(),
                Some(ThreeAddressInstruction::Terminate)
            ),
            "expected terminate as last instruction, so that the function returns"
        );

        self.asm.machine_code()
    }

    // STACK
    //
    // $sp == $sp + 0x00 [previous x20]
    //        $sp + 0x08 [previous x21]
    //        $sp + 0x10 [previous x19]
    //        $sp + 0x18 [ ...unused  ]
    // $fp == $sp + 0x20 [previous  fp] | Frame record
    //        $sp + 0x28 [previous  lr] |

    // REGISTERS
    //
    // x19 <- pointer into the universe
    // x20 <- pointer to putchar()
    // x21 <- pointer to getchar()

    fn setup_stack_and_save_registers(&mut self) {
        //  stp	x20, x21, [sp, #-0x30]!
        //  stp x29, x30, [sp, #0x20]
        //  str	x19, [sp, 0x10]
        self.asm.stp_preindex(PUTCHAR, GETCHAR, SP, -0x30);
        self.asm.stp_offset(FP, LR, SP, 0x20);
        self.asm.str_imm(ADDR, SP, 0x10);

        // Let the frame pointer point to the current frame record
        // -- this allows backtraces to work, since the frame pointer,
        //    and all the frame records is a linked-list of stack frames
        self.asm.add64(FP, SP, 0x20);

        // mov x19, x0
        // mov x20, x1
        // mov x21, x2
        self.asm.mov(ADDR, X(0));
        self.asm.mov(PUTCHAR, X(1));
        self.asm.mov(GETCHAR, X(2));
    }

    fn restore_stack_and_registers_and_return(&mut self) {
        // ldr x19, [sp, #0x10]
        // ldp x29, x30 [sp, #0x20]
        // ldp x20, x21 [sp], #0x30
        self.asm.ldr_imm(ADDR, SP, 0x10);
        self.asm.ldp_offset(FP, LR, SP, 0x20);
        self.asm.ldp_postindex(PUTCHAR, GETCHAR, SP, 0x30);
        self.asm.ret();
    }

    fn generate_code(&mut self, cfg: &ControlFlowGraph) {
        // First-pass: generate instructions, but branches will be incomplete.
        for block in cfg.blocks().iter() {
            let BlockLabel(l) = block.label();
            self.asm.set_label_target(Label(l));
            for &instr in block.instructions().iter() {
                self.generate_instructions(instr);
            }
        }

        // Second-pass: patch all incomplete instructions
        self.asm.patch_branch_targets();
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
                self.asm.cbz(VAL, Label(l));
            }
            BranchTo(BlockLabel(l)) => {
                // b    L*
                self.asm.b(Label(l));
            }
            Terminate => {
                self.restore_stack_and_registers_and_return();
            }
        }
    }
}
