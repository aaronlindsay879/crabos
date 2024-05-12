global long_mode_start:function

extern kernel_main

section .text
bits 64
long_mode_start:
	mov rdi, rbx
	call kernel_main

	cli
.hang   hlt
	jmp .hang
