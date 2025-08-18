use core::arch::naked_asm;
use loongArch64::register::badv;

use crate::{println, trap::TrapContext};

macro_rules! includes_trap_macros {
    () => {
        r#"
        .ifndef REGS_TRAP_MACROS_FLAG
        .equ REGS_TRAP_MACROS_FLAG, 1

        // 2, 4, 1
        .macro FIXUP_EX from, to, fix
        .if \fix
            .section .fixup, "ax"
        \to: 
            li.w	$a0, -1
            jr	$ra
            .previous
        .endif
            .section __ex_table, "a"
            .word	\from\()b, \to\()b
            .previous
        .endm

        .equ KSAVE_KSP,  0x30
        .equ KSAVE_CTX,  0x31
        .equ KSAVE_USP,  0x32
        .equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
        .equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
        .equ LA_CSR_PGD,           0x1b    /* Page table base */
        .equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
        .equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
        .equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
        .equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
        .equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
        .equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
        .equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */
        .macro SAVE_REGS
            st.d    $ra, $sp,  1*8
            st.d    $tp, $sp,  2*8
            st.d    $a0, $sp,  4*8
            st.d    $a1, $sp,  5*8
            st.d    $a2, $sp,  6*8
            st.d    $a3, $sp,  7*8
            st.d    $a4, $sp,  8*8
            st.d    $a5, $sp,  9*8
            st.d    $a6, $sp, 10*8
            st.d    $a7, $sp, 11*8
            st.d    $t0, $sp, 12*8
            st.d    $t1, $sp, 13*8
            st.d    $t2, $sp, 14*8
            st.d    $t3, $sp, 15*8
            st.d    $t4, $sp, 16*8
            st.d    $t5, $sp, 17*8
            st.d    $t6, $sp, 18*8
            st.d    $t7, $sp, 19*8
            st.d    $t8, $sp, 20*8
            st.d    $r21,$sp, 21*8
            st.d    $fp, $sp, 22*8
            st.d    $s0, $sp, 23*8
            st.d    $s1, $sp, 24*8
            st.d    $s2, $sp, 25*8
            st.d    $s3, $sp, 26*8
            st.d    $s4, $sp, 27*8
            st.d    $s5, $sp, 28*8
            st.d    $s6, $sp, 29*8
            st.d    $s7, $sp, 30*8
            st.d    $s8, $sp, 31*8
            csrrd   $t0, KSAVE_USP
            st.d    $t0, $sp,  3*8

            csrrd	$t0, 0x1
            st.d	$t0, $sp, 8*32  // prmd

            csrrd   $t0, 0x6        
            st.d    $t0, $sp, 8*33  // era
        .endm

        .macro LOAD_REGS
            ld.d    $t0, $sp, 32*8
            csrwr   $t0, 0x1        // Write PRMD(PLV PIE PWE) to prmd

            ld.d    $t0, $sp, 33*8
            csrwr   $t0, 0x6        // Write Exception Address to ERA

            ld.d    $ra, $sp, 1*8
            ld.d    $tp, $sp, 2*8
            ld.d    $a0, $sp, 4*8
            ld.d    $a1, $sp, 5*8
            ld.d    $a2, $sp, 6*8
            ld.d    $a3, $sp, 7*8
            ld.d    $a4, $sp, 8*8
            ld.d    $a5, $sp, 9*8
            ld.d    $a6, $sp, 10*8
            ld.d    $a7, $sp, 11*8
            ld.d    $t0, $sp, 12*8
            ld.d    $t1, $sp, 13*8
            ld.d    $t2, $sp, 14*8
            ld.d    $t3, $sp, 15*8
            ld.d    $t4, $sp, 16*8
            ld.d    $t5, $sp, 17*8
            ld.d    $t6, $sp, 18*8
            ld.d    $t7, $sp, 19*8
            ld.d    $t8, $sp, 20*8
            ld.d    $r21,$sp, 21*8
            ld.d    $fp, $sp, 22*8
            ld.d    $s0, $sp, 23*8
            ld.d    $s1, $sp, 24*8
            ld.d    $s2, $sp, 25*8
            ld.d    $s3, $sp, 26*8
            ld.d    $s4, $sp, 27*8
            ld.d    $s5, $sp, 28*8
            ld.d    $s6, $sp, 29*8
            ld.d    $s7, $sp, 30*8
            ld.d    $s8, $sp, 31*8
            
            // restore sp
            ld.d    $sp, $sp, 3*8
        .endm

        .endif
        "#
    }
}

pub const LDH_OP: u32 = 0xa1;
pub const LDHU_OP: u32 = 0xa9;
pub const LDW_OP: u32 = 0xa2;
pub const LDWU_OP: u32 = 0xaa;
pub const LDD_OP: u32 = 0xa3;
pub const STH_OP: u32 = 0xa5;
pub const STW_OP: u32 = 0xa6;
pub const STD_OP: u32 = 0xa7;

pub const LDPTRW_OP: u32 = 0x24;
pub const LDPTRD_OP: u32 = 0x26;
pub const STPTRW_OP: u32 = 0x25;
pub const STPTRD_OP: u32 = 0x27;

pub const LDXH_OP: u32 = 0x7048;
pub const LDXHU_OP: u32 = 0x7008;
pub const LDXW_OP: u32 = 0x7010;
pub const LDXWU_OP: u32 = 0x7050;
pub const LDXD_OP: u32 = 0x7018;
pub const STXH_OP: u32 = 0x7028;
pub const STXW_OP: u32 = 0x7030;
pub const STXD_OP: u32 = 0x7038;

#[allow(binary_asm_labels)]
#[naked]
unsafe extern "C" fn unaligned_read(addr: u64, value: &mut u64, n: u64, symbol: u32) -> i32 {
    naked_asm!(
        includes_trap_macros!(),
        "
            beqz	$a2, 5f

            li.w	$t1, 8
            li.w	$t2, 0

            addi.d	$t0, $a2, -1
            mul.d	$t1, $t0, $t1
            add.d 	$a0, $a0, $t0

            beq	    $a3, $zero, 2f
        1:	ld.b	$t3, $a0, 0
            b	3f

        2:	ld.bu	$t3, $a0, 0
        3:	sll.d	$t3, $t3, $t1
            or	    $t2, $t2, $t3
            addi.d	$t1, $t1, -8
            addi.d	$a0, $a0, -1
            addi.d	$a2, $a2, -1
            bgt	    $a2, $zero, 2b
        4:	st.d	$t2, $a1, 0

            move	$a0, $a2
            jr	    $ra

        5:	li.w    $a0, -1
            jr	    $ra

            FIXUP_EX 1, 6, 1
            FIXUP_EX 2, 6, 0
            FIXUP_EX 4, 6, 0
        ",
    )
}

#[allow(binary_asm_labels)]
#[naked]
unsafe extern "C" fn unaligned_write(_addr: u64, _value: u64, _n: u64) -> i32 {
    naked_asm!(
        includes_trap_macros!(),
        "
        beqz	$a2, 3f

        li.w	$t0, 0
    1:	srl.d	$t1, $a1, $t0
    2:	st.b	$t1, $a0, 0
        addi.d	$t0, $t0, 8
        addi.d	$a2, $a2, -1
        addi.d	$a0, $a0, 1
        bgt	    $a2, $zero, 1b
    
        move	$a0, $a2
        jr	    $ra
    
    3:	li.w    $a0, -1
        jr	    $ra
    
        FIXUP_EX 2, 4, 1
        ",
    )
}

#[unsafe(export_name = "kernel_ale_handler")]
pub fn kernel_ale_handler(ctx: &mut TrapContext) {
    unsafe { emulate_load_store_insn(ctx) }
}

#[allow(unused_assignments)]
pub unsafe fn emulate_load_store_insn(ctx: &mut TrapContext) {
    let la_inst: u32;
    let addr: u64;
    let rd: usize;

    let mut value: u64 = 0;
    let mut res: i32 = 0;

    unsafe {
        core::arch::asm!(
            "ld.w {val}, {addr}, 0 ",
             addr = in(reg) ctx.era as u64,
             val = out(reg) la_inst,
        )
    }
    addr = badv::read().vaddr() as u64;
    rd = (la_inst & 0x1f) as usize;

    if (la_inst >> 22) == LDD_OP || (la_inst >> 24) == LDPTRD_OP || (la_inst >> 15) == LDXD_OP {
        res = unaligned_read(addr, &mut value, 8, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        ctx.r[rd] = value as usize;
    } else if (la_inst >> 22) == LDW_OP
        || (la_inst >> 24) == LDPTRW_OP
        || (la_inst >> 15) == LDXW_OP
    {
        res = unaligned_read(addr, &mut value, 4, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        ctx.r[rd] = value as usize;
    } else if (la_inst >> 22) == LDWU_OP || (la_inst >> 15) == LDXWU_OP {
        res = unaligned_read(addr, &mut value, 4, 0);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        ctx.r[rd] = value as usize;
    } else if (la_inst >> 22) == LDH_OP || (la_inst >> 15) == LDXH_OP {
        res = unaligned_read(addr, &mut value, 2, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        ctx.r[rd] = value as usize;
    } else if (la_inst >> 22) == LDHU_OP || (la_inst >> 15) == LDXHU_OP {
        res = unaligned_read(addr, &mut value, 2, 0);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        ctx.r[rd] = value as usize;
    } else if (la_inst >> 22) == STD_OP
        || (la_inst >> 24) == STPTRD_OP
        || (la_inst >> 15) == STXD_OP
    {
        value = ctx.r[rd] as u64;
        res = unaligned_write(addr, value, 8);
        // write_bytes(addr, value, 8);
    } else if (la_inst >> 22) == STW_OP
        || (la_inst >> 24) == STPTRW_OP
        || (la_inst >> 15) == STXW_OP
    {
        value = ctx.r[rd] as u64;
        res = unaligned_write(addr, value, 4);
    } else if (la_inst >> 22) == STH_OP || (la_inst >> 15) == STXH_OP {
        value = ctx.r[rd] as u64;
        res = unaligned_write(addr, value, 2);
    } else {
        panic!("unhandled unaligned address, inst:{:#x}", la_inst);
    }

    if res < 0 {
        panic!("Address Error @ {:#x}", addr)
    }

    ctx.era += 4;
}