    .section .text
    .align 12
    .global sigreturn_trampoline
sigreturn_trampoline:
    li.d $a7, 139
    syscall 0