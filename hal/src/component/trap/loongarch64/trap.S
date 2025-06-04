.altmacro

.equ CRMD,  0x0
.equ PRMD,  0x1
.equ ESTAT, 0x5
.equ ERA,   0x6
.equ BADV,  0x7
.equ BADI,  0x8
.equ KSAVE_CTX,  0x30

.macro SAVE_GP n
    st.d $r\n, $sp, \n*8
.endm
.macro LOAD_GP n
    ld.d $r\n, $sp, \n*8
.endm
    .section .text
    .globl __trap_from_user
    .globl __restore
    .globl __trap_from_kernel
    .globl __user_rw_exception_entry
    .globl __user_rw_trap_vector
    .align 12
__trap_from_user:
    csrwr $sp, KSAVE_CTX
    # now sp->*TrapContext in user space, KSAVE_CTX->user stack
    # save other general purpose registers

    # save ra
    st.d $r1, $sp, 1*8
    # save tp
    st.d $r2, $sp, 2*8

    # skip sp(r3), we will save it later

    # save r4~r31
    .set n, 4
    .rept 28
        SAVE_GP %n
        .set n, n+1
    .endr

    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrrd $t0, PRMD
    csrrd $t1, ERA
    st.d $t0, $sp, 32*8
    st.d $t1, $sp, 33*8
    # read user stack from KSAVE_CTX and save it in TrapContext
    csrrd $t2, KSAVE_CTX
    st.d $t2, $sp, 3*8

    # move to kernel_sp
    # Load callee-saved registers (s0-s11)
    ld.d $ra, $sp, 35*8
    ld.d $s0, $sp, 36*8
    ld.d $s1, $sp, 37*8
    ld.d $s2, $sp, 38*8
    ld.d $s3, $sp, 39*8
    ld.d $s4, $sp, 40*8
    ld.d $s5, $sp, 41*8
    ld.d $s6, $sp, 42*8
    ld.d $s7, $sp, 43*8
    ld.d $s8, $sp, 44*8
    ld.d $fp, $sp, 45*8
    ld.d $tp, $sp, 46*8

    # set kernel_sp
    ld.d $sp, $sp, 34*8
    # jump to trap_handler
    ret
    nop

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    csrwr $a0, KSAVE_CTX
    csrrd $a0, KSAVE_CTX
    # save kernel callee-saved regs
    st.d $sp, $a0, 34*8
    st.d $ra, $a0, 35*8
    st.d $s0, $a0, 36*8
    st.d $s1, $a0, 37*8
    st.d $s2, $a0, 38*8
    st.d $s3, $a0, 39*8
    st.d $s4, $a0, 40*8
    st.d $s5, $a0, 41*8
    st.d $s6, $a0, 42*8
    st.d $s7, $a0, 43*8
    st.d $s8, $a0, 44*8
    st.d $fp, $a0, 45*8
    st.d $tp, $a0, 46*8
    move $sp, $a0

    # restore prmd/era
    ld.d $t0, $sp, 32*8
    ld.d $t1, $sp, 33*8
    csrwr $t0, PRMD
    csrwr $t1, ERA

    # restore general purpose registers except x0/sp/tp
    # load ra
    ld.d $r1, $sp, 1*8
    # load tp
    ld.d $r2, $sp, 2*8

    # load r4~r31
    .set n, 4
    .rept 28
        LOAD_GP %n
        .set n, n+1
    .endr

    # back to user stack
    ld.d $sp, $sp, 3*8
    ertn

    .align 12
__trap_from_kernel:
    # need to save caller-saved regs
    # note that we need save ERA & BADV here
    addi.d $sp, $sp, -20*8
    st.d $ra, $sp, 0*8
    st.d $tp, $sp, 1*8
    st.d $a0, $sp, 2*8
    st.d $a1, $sp, 3*8
    st.d $a2, $sp, 4*8
    st.d $a3, $sp, 5*8
    st.d $a4, $sp, 6*8
    st.d $a5, $sp, 7*8
    st.d $a6, $sp, 8*8
    st.d $a7, $sp, 9*8
    st.d $t0, $sp, 10*8
    st.d $t1, $sp, 11*8
    st.d $t2, $sp, 12*8
    st.d $t3, $sp, 13*8
    st.d $t4, $sp, 14*8
    st.d $t5, $sp, 15*8
    st.d $t6, $sp, 16*8
    st.d $t7, $sp, 17*8
    st.d $t8, $sp, 18*8
    st.d $r21, $sp, 19*8
    bl kernel_trap_handler
    ld.d $ra, $sp, 0*8
    ld.d $tp, $sp, 1*8
    ld.d $a0, $sp, 2*8
    ld.d $a1, $sp, 3*8
    ld.d $a2, $sp, 4*8
    ld.d $a3, $sp, 5*8
    ld.d $a4, $sp, 6*8
    ld.d $a5, $sp, 7*8
    ld.d $a6, $sp, 8*8
    ld.d $a7, $sp, 9*8
    ld.d $t0, $sp, 10*8
    ld.d $t1, $sp, 11*8
    ld.d $t2, $sp, 12*8
    ld.d $t3, $sp, 13*8
    ld.d $t4, $sp, 14*8
    ld.d $t5, $sp, 15*8
    ld.d $t6, $sp, 16*8
    ld.d $t7, $sp, 17*8
    ld.d $t8, $sp, 18*8
    ld.d $r21, $sp, 19*8
    addi.d $sp, $sp, 20*8
    ertn


__user_rw_exception_entry:
    csrrd   $a0, ERA
    addi.d  $a0, $a0, 4
    csrwr   $a0, ERA
    csrrd   $a1, ESTAT
    srli.d  $a1, $a1, 16
    andi    $a1, $a1, 0x3f
    move    $a0, $zero
    ertn

# 在计算中断的例外入口时，中断对应的例外号是中断号+64
    .align 12
__user_rw_trap_vector:
    nop
    nop
    b __user_rw_exception_entry
    nop
    b __user_rw_exception_entry
    nop
    b __trap_from_kernel
    nop
    b __user_rw_exception_entry
    nop
    .rept 123
        b __trap_from_kernel
        nop
    .endr
