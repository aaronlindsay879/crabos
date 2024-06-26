# Crabos
Simple OS written (mostly) in rust.

Use the provided makefile to run.

Project structure:
* [crabstd](crabstd) - standard library
* [drivers](drivers) - set of device and file system drivers
* [kernel](kernel) - core kernel code
* [kernel_loader](kernel_loader) - loader for kernel, sets up higher half memory
* [kernel_shared](kernel_shared) - code that's shared between core kernel and loader
* [multiboot](multiboot) - multiboot2 header and boot information library
* [x86_64](x86_64) - various x86_64 specific instructions, with the goal of abstracting away as much inline assembly as possible
* [generate_initrd.py](generate_initrd.py) - simple script for generating `initrd` file for kernel (currently filled with arbitrary data for testing)