.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __trap_from_user
    .globl __restore
    .globl __trap_from_kernel
    .globl __user_rw_exception_entry
    .globl __user_rw_trap_vector
    .align 2
__trap_from_user:
    csrrw sp, sscratch, sp
    # now sp->TrapContext, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skipp /x $ sp(x2), we will save it later
    
    # save x3~x31 (x4 is tp)
    .set n, 3
    .rept 29
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # move to kernel_sp
    # Load callee-saved registers (s0-s11)
    ld ra, 35*8(sp)
    ld s0, 36*8(sp)
    ld s1, 37*8(sp)
    ld s2, 38*8(sp)
    ld s3, 39*8(sp)
    ld s4, 40*8(sp)
    ld s5, 41*8(sp)
    ld s6, 42*8(sp)
    ld s7, 43*8(sp)
    ld s8, 44*8(sp)
    ld s9, 45*8(sp)
    ld s10, 46*8(sp)
    ld s11, 47*8(sp)
    ld fp, 48*8(sp)
    ld tp, 49*8(sp)
    # set kernel_sp
    ld sp, 34*8(sp)
    # jump to trap_handler
    ret 

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    csrw sscratch, a0
    # save kernel callee-saved regs
    sd sp, 34*8(a0)
    sd ra, 35*8(a0)
    sd s0, 36*8(a0)
    sd s1, 37*8(a0)
    sd s2, 38*8(a0)
    sd s3, 39*8(a0)
    sd s4, 40*8(a0)
    sd s5, 41*8(a0)
    sd s6, 42*8(a0)
    sd s7, 43*8(a0)
    sd s8, 44*8(a0)
    sd s9, 45*8(a0)
    sd s10, 46*8(a0)
    sd s11, 47*8(a0)
    sd fp, 48*8(a0)
    sd tp, 49*8(a0)
    mv sp, a0
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret

__trap_from_kernel:
    # need to save caller-saved regs
    sd  t0, -17*8(sp)
    sd  t1, -16*8(sp)
    csrr t0, sepc
    csrr t1, sstatus
    sd  t0, -19*8(sp)
    sd  t1, -18*8(sp)
    sd  t2, -15*8(sp)
    sd  t3, -14*8(sp)
    sd  t4, -13*8(sp)
    sd  t5, -12*8(sp)
    sd  t6, -11*8(sp)
    sd  a0, -10*8(sp)
    sd  a1, -9*8(sp)
    sd  a2, -8*8(sp)
    sd  a3, -7*8(sp)
    sd  a4, -6*8(sp)
    sd  a5, -5*8(sp)
    sd  a6, -4*8(sp)
    sd  a7, -3*8(sp)
    # save fp for backtrace
    sd  fp, -2*8(sp)
    sd  ra, -1*8(sp)
    mv  fp, sp
    # sp must align to 16
    addi sp, sp, -20*8
    call kernel_trap_handler
    ld  t0,  1*8(sp)
    ld  t1,  2*8(sp)
    csrw sepc,    t0
    csrw sstatus, t1
    ld  t0,  3*8(sp)
    ld  t1,  4*8(sp)
    ld  t2,  5*8(sp)
    ld  t3,  6*8(sp)
    ld  t4,  7*8(sp)
    ld  t5,  8*8(sp)
    ld  t6,  9*8(sp)
    ld  a0, 10*8(sp)
    ld  a1, 11*8(sp)
    ld  a2, 12*8(sp)
    ld  a3, 13*8(sp)
    ld  a4, 14*8(sp)
    ld  a5, 15*8(sp)
    ld  a6, 16*8(sp)
    ld  a7, 17*8(sp)
    ld  fp, 18*8(sp)
    ld  ra, 19*8(sp)
    addi sp, sp, 20*8
    sret

__user_rw_exception_entry:
    csrr a0, sepc
    addi a0, a0, 4
    csrw sepc, a0
    mv   a0, zero
    csrr a1, scause
    sret

# When MODE=Vectored, all synchronous exceptions into supervisor mode 
# cause the pc to be set to the address in the BASE field, whereas
# interrupts cause the pc to be set to the address in the BASE field 
# plus four times the interrupt cause number.
    .align 8
__user_rw_trap_vector:
    j __user_rw_exception_entry
    .rept 16
    .align 2
    j __trap_from_kernel
    .endr
    unimp
