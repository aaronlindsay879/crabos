AS := nasm
ASFLAGS := -felf64

PROJDIRS := kernel kernel_loader kernel_shared crabstd x86_64 multiboot drivers

RUST_SRC_FILES := $(shell find $(PROJDIRS) -type f -name "*.rs")

ASM_SRC_DIR := kernel_loader/src/arch/x86_64/asm
ASM_OBJ_DIR := target/asm

ASM_SRC_FILES := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJ_FILES := $(patsubst %.asm, target/asm/%.o, $(notdir $(ASM_SRC_FILES)))

GRUB_FILES := kernel_loader/src/arch/x86_64/boot
LINKER_SCRIPTS := $(shell find kernel_loader/ -type f -name "*.ld")

BIN_FILE := target/isofiles/boot/crabos
LOADER_FILE = target/isofiles/boot/crabos-loader
INITRD_FILE := target/isofiles/boot/crabos.initrd

LIB_FILE := target/x86_64-unknown-crabos/release/libcrabos.a
LOADER_LIB_FILE := target/x86_64-unknown-crabos/release/libkernel_loader.a

ISO_FILE := target/crabos.iso

run: $(ISO_FILE)
	qemu-system-x86_64 \
				-drive file=$(ISO_FILE),format=raw \
				-display gtk,show-tabs=on -m 256M \
				-serial stdio

clean: 
	cargo clean

$(ISO_FILE): $(BIN_FILE) $(LOADER_FILE) $(INITRD_FILE) $(wildcard $(GRUB_FILES)/**/*)
	cp -r $(GRUB_FILES)/ target/isofiles

	grub-mkrescue -o $(ISO_FILE) target/isofiles

$(INITRD_FILE): generate_initrd.py
	python generate_initrd.py $(INITRD_FILE)

$(BIN_FILE): $(RUST_SRC_FILES) kernel/layout.ld
	cargo build --release --package crabos
	mkdir -p target/isofiles/boot
	ld -n --no-warn-rwx-segment \
		-Tkernel/layout.ld -o $(BIN_FILE) \
		$(LIB_FILE)

$(LOADER_FILE): $(LOADER_LIB_FILE) $(ASM_OBJ_FILES) $(LINKER_SCRIPTS)
	mkdir -p target/isofiles/boot
	ld -n --gc-sections --no-warn-rwx-segment \
		$(addprefix -T, $(LINKER_SCRIPTS)) -o $(LOADER_FILE) \
		$(ASM_OBJ_FILES) $(LOADER_LIB_FILE)

$(LOADER_LIB_FILE): $(RUST_SRC_FILES)
	cargo build --release --package kernel_loader

$(ASM_OBJ_DIR)/%.o: $(ASM_SRC_DIR)/%.asm
	mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) $< -o $@