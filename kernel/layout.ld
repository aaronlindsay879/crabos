/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(kernel_main)
 
/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	. = 0xffffffff80000000;
 
	.boot BLOCK(4K) : ALIGN(4K) {
		KEEP(*(.multiboot))
	}

	.text BLOCK(4K) : ALIGN(4K)
	{
		*(.text .text.*)
	}

	.syscall_table BLOCK(4K) : ALIGN(4K) 
	{
		*(.syscall_table)
	}
 
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata .rodata.*)
	}
 
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data .data.*)
	}
 
	.bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss .bss.*)
	}
 
	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */

	/DISCARD/ :
	{
		*(.comment)
	}
}