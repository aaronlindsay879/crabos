global set_up_page_tables:function
global enable_paging:function

extern p4_table
extern p3_table
extern p3_table_phys

section .text
bits 32
set_up_page_tables:
	; map first P4 entry to P3 table
	mov eax, p3_table 
	or eax, 0b11 ; present + writable
	mov [p4_table], eax

	; also point to physical memory mappings
	mov eax, p3_table_phys
	or eax, 0b11
	mov [p4_table + 256 * 8], eax

	; identity map first 1GiB for kernel loader
	; 0b10000011 = present + writable + huge
	mov dword [p3_table], 0x00000000 | 0b10000011

	; also map first 4GiB of physical memory in proper location 
	mov dword [p3_table_phys + 0 * 8], 0x00000000 | 0b10000011
	mov dword [p3_table_phys + 1 * 8], 0x40000000 | 0b10000011
	mov dword [p3_table_phys + 2 * 8], 0x80000000 | 0b10000011
	mov dword [p3_table_phys + 3 * 8], 0xC0000000 | 0b10000011

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
