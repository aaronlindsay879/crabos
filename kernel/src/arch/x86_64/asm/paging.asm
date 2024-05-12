global set_up_page_tables:function
global enable_paging:function

extern p4_table
extern p3_table
extern p2_table

section .text
bits 32
set_up_page_tables:
	; recursively map p4 table
	mov eax, p4_table
	or eax, 0b11 ; present + writable
	mov [p4_table + 511 * 8], eax

	; map first P4 entry to P3 table
	mov eax, p3_table 
	or eax, 0b11 ; present + writable
	mov [p4_table], eax

	; map first P3 entry to P2 table
	mov eax, p2_table
	or eax, 0b11 ; present + writable
	mov [p3_table], eax

	; map each P2 entry to huge 2 MiB page
	mov ecx, 0 ; counter

.map_p2_table:
	mov eax, 0x200000 ; 2 MiB
	mul ecx ; start address
	or eax, 0b10000011 ; present + writable + huge
	mov [p2_table + ecx * 8], eax ; map entry

	inc ecx
	cmp ecx, 512
	jne .map_p2_table ; map next entry if counter not 512 yet

	ret

enable_paging:
	; load P4 to cr3 register
	mov eax, p4_table
	mov cr3, eax

	; enable PAE flag in cr4
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; set long mode and NXE bit in model specific register
	mov ecx, 0xC0000080
	rdmsr
	or eax, (1 << 11) | (1 << 8)
	wrmsr

	; enabling paging and write protect in cr0 register
	mov eax, cr0
	or eax, (1 << 31) | (1 << 16)
	mov cr0, eax

	ret
