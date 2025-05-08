    .section .text
    .global sigreturn_trampoline
    .align 12
sigreturn_trampoline:
    li.d $a7, 139
    syscall 0