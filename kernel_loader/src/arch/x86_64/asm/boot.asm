global start:function
global p4_table:data
global p3_table:data
global p3_table_phys:data

section .bss
align 4096
p4_table: 
	resb 4096
p3_table:
	resb 4096
p3_table_phys: 
	resb 4096
stack_bottom:
	resb 4096 * 16
stack_top:

section .rodata
gdt64:  ; set up GDT in 64 bit mode
	dq 0 ; zero entry
.code:  equ $ - gdt64
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
	dw $ - gdt64 - 1
	dq gdt64

section .text
bits 32
start:
	; set up stack and save value of ebx for booting
	mov esp, stack_top
	push ebx

	; make sure we booted from a multiboot bootloader, we have cpuid support, and can boot into long mode
	extern check_multiboot
	extern check_cpuid
	extern check_long_mode

	call check_multiboot
	call check_cpuid
	call check_long_mode

	; set up page tables and then enable paging
	extern set_up_page_tables
	extern enable_paging

	call set_up_page_tables
	call enable_paging

	; finally start in long mode and restore value of ebx
	extern long_mode_start

	pop ebx
	lgdt [gdt64.pointer]
	jmp gdt64.code:long_mode_start
.end: