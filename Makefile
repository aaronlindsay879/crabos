AS := nasm
ASFLAGS := -felf64

PROJDIRS := kernel crabstd x86_64 multiboot drivers

RUST_SRC_FILES := $(shell find $(PROJDIRS) -type f -name "*.rs")

ASM_SRC_DIR := kernel/src/arch/x86_64/asm
ASM_OBJ_DIR := target/asm

ASM_SRC_FILES := $(wildcard $(ASM_SRC_DIR)/*.asm)
ASM_OBJ_FILES := $(patsubst %.asm, target/asm/%.o, $(notdir $(ASM_SRC_FILES)))

GRUB_FILES := kernel/src/arch/x86_64/boot
LINKER_SCRIPTS := $(shell find $(PROJDIRS) -type f -name "*.ld")

BIN_FILE := target/isofiles/boot/crabos.bin
INITRD_FILE := target/isofiles/boot/crabos.initrd
LIB_FILE := target/x86_64-unknown-crabos/release/libcrabos.a
ISO_FILE := target/crabos.iso

QEMU_BASE := qemu-system-x86_64 \
				-drive file=$(ISO_FILE),format=raw \
				-display gtk,show-tabs=on \
				-serial stdio

bios: $(ISO_FILE)
	$(QEMU_BASE)

uefi: $(ISO_FILE)
	$(QEMU_BASE) \
		-drive if=pflash,format=raw,unit=0,file=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd,readonly=on \
		-drive if=pflash,format=raw,unit=1,file=OVMF_VARS.fd

clean: 
	cargo clean

$(ISO_FILE): $(BIN_FILE) $(INITRD_FILE) $(wildcard $(GRUB_FILES)/**/*)
	cp -r $(GRUB_FILES)/ target/isofiles

	grub-mkrescue -o $(ISO_FILE) target/isofiles

$(INITRD_FILE): generate_initrd.py
	python generate_initrd.py $(INITRD_FILE)

$(BIN_FILE): $(LIB_FILE) $(ASM_OBJ_FILES) $(LINKER_SCRIPTS)
	mkdir -p target/isofiles/boot
	ld -n --gc-sections --no-warn-rwx-segment \
		$(addprefix -T, $(LINKER_SCRIPTS)) -o $(BIN_FILE) \
		$(ASM_OBJ_FILES) $(LIB_FILE)

$(LIB_FILE): $(RUST_SRC_FILES)
	cargo build --release

$(ASM_OBJ_DIR)/%.o: $(ASM_SRC_DIR)/%.asm
	mkdir -p $(shell dirname $@)
	$(AS) $(ASFLAGS) $< -o $@