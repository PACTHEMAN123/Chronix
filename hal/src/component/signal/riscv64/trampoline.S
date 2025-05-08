    .section .text
    .global sigreturn_trampoline
    .align 12
sigreturn_trampoline:
    li a7, 139
    ecall