## Booting process
* GRUB invokes [assembly trampoline](/kernel_loader/src/arch/x86_64/asm)
  * Sets up basic stack
  * Identity maps first 1GiB and maps first 4GiB of memory to 0xffff800000000000
  * Enables 64 bit mode and jumps to `loader_main` in [kernel_loader](/kernel_loader/src/lib.rs)
* Kernel loader is invoked by assembly trampoline
  * Initialises frame allocator
  * Maps memory according to [the memory map](MemoryMap.md)
  * Hands control over to kernel