# Crabos
Simple OS written (mostly) in rust.

Use the provided makefile to run, with `uefi` and `bios` targets for running in `uefi` and `bios` mode.

Project structure:
* [crabstd](crabstd) - standard library
* [drivers](drivers) - set of device and file system drivers
* [kernel](kernel) - core kernel code
* [multiboot](multiboot) - multiboot2 header and boot information library
* [x86_64](x86_64) - various x86_64 specific instructions, with the goal of abstracting away as much inline assembly as possible
* [generate_initrd.py](generate_initrd.py) - simple script for generating `initrd` file for kernel (currently filled with arbitrary data for testing)