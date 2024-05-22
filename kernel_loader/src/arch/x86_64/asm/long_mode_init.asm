global long_mode_start:function

extern loader_main

section .text
bits 64
long_mode_start:
	mov rdi, rbx
	call loader_main

	cli
.hang   hlt
	jmp .hang
