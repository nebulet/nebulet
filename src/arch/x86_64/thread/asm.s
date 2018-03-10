.section .text
.intel_syntax noprefix
.code64
.global x86_64_context_switch

# Expected Arguments
# - Pointer to location to store previous stack pointer in  `rdi`
# - New stack pointer in `rsi`
x86_64_context_switch:
    pushfq
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    mov [rdi], rsp
    mov rsp, rsi

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    popfq

    ret