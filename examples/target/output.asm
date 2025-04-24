section .data
result: dq 0x40866666
i: dq -10
str_0: db "End of program.", 0
str_1: db "End of program.", 0
buffer: times 32 db 0
newline: db 10, 0
input_buffer: times 256 db 0
float_format: db "%f", 0
section .bss
n resq 8
A resq 40
section .text
global _start
_start:
    push rbp
    mov rbp, rsp
    sub rsp, 1024
push rax
push rbx
push rcx
mov rax, 0
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x3f800000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 1
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40000000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 2
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40400000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 3
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40800000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 4
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40accccd
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
mov rax, 5
mov [n], rax
mov rax, 0x3f800000
mov [result], rax
push rax
push rbx
push rcx
mov rax, 2
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40866666
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
mov rax, [result]
push rax
call print_int
pop rax
lea rax, [str_0]
push rax
call print_string
pop rax
push rax
push rbx
push rcx
mov rax, 0
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x3f800000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 1
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40000000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 2
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40400000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 3
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40800000
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
push rax
push rbx
push rcx
mov rax, 4
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40accccd
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
mov rax, 5
mov [n], rax
mov rax, 0x3f800000
mov [result], rax
push rax
push rbx
push rcx
mov rax, 2
mov rbx, 8
imul rax, rbx
mov rbx, A
mov rcx, 0x40866666
mov [rbx+rax], rcx
pop rcx
pop rbx
pop rax
mov rax, [result]
push rax
call print_int
pop rax
lea rax, [str_1]
push rax
call print_string
pop rax
    mov rax, 60
    xor rdi, rdi
    syscall

; Function to print integers
print_int:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    mov rax, [rsp+40]
    mov rcx, 10
    mov rbx, buffer+31
    mov byte [rbx], 0
    dec rbx
    mov r12, 0
    cmp rax, 0
    jge .positive
    mov r12, 1
    neg rax
.positive:
.loop:
    xor rdx, rdx
    div rcx
    add dl, '0'
    mov [rbx], dl
    dec rbx
    test rax, rax
    jnz .loop
    cmp r12, 1
    jne .print
    mov byte [rbx], '-'
    dec rbx
.print:
    inc rbx
    mov rax, 1
    mov rdi, 1
    mov rsi, rbx
    mov rdx, buffer+31
    sub rdx, rbx
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

; Function to read integers
read_int:
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    mov rax, 0
    mov rdi, 0
    mov rsi, input_buffer
    mov rdx, 255
    syscall
    mov rcx, 0
    mov rbx, input_buffer
    mov r12, 0
    cmp byte [rbx], '-'
    jne .parse_loop
    mov r12, 1
    inc rbx
.parse_loop:
    movzx rax, byte [rbx]
    cmp al, 10
    je .parse_done
    cmp al, 0
    je .parse_done
    sub al, '0'
    imul rcx, 10
    add rcx, rax
    inc rbx
    jmp .parse_loop
.parse_done:
    cmp r12, 1
    jne .return
    neg rcx
.return:
    mov rax, rcx
    pop r12
    pop rbx
    pop rbp
    ret

; Function to print floats
print_float:
    push rbp
    mov rbp, rsp
    fstp qword [rsp-8]
    sub rsp, 8
    fld qword [rsp]
    lea rbx, [buffer]
    fld st0
    frndint
    fistp qword [rbx]
    mov rax, [rbx]
    push rax
    call print_float_helper
    add rsp, 8
    add rsp, 8
    pop rbp
    ret
print_float_helper:
    push rbp
    mov rbp, rsp
    mov rax, [rsp+16]
    push rax
    call print_int
    add rsp, 8
    pop rbp
    ret

; Function to read floats
read_float:
    push rbp
    mov rbp, rsp
    call read_int
    cvtsi2sd xmm0, rax
    pop rbp
    ret

; Function to print strings
print_string:
    push rbp
    mov rbp, rsp
    push rbx
    mov rbx, [rsp+24]
    mov rdx, 0
.strlen_loop:
    cmp byte [rbx+rdx], 0
    je .print_it
    inc rdx
    jmp .strlen_loop
.print_it:
    mov rax, 1
    mov rdi, 1
    mov rsi, rbx
    syscall
    pop rbx
    pop rbp
    ret

; Function to read strings
read_string:
    push rbp
    mov rbp, rsp
    mov rsi, [rsp+16]
    mov rdx, [rsp+24]
    mov rax, 0
    mov rdi, 0
    syscall
    mov rbx, rsi
    add rbx, rax
    dec rbx
    cmp byte [rbx], 10
    jne .done
    mov byte [rbx], 0
.done:
    pop rbp
    ret