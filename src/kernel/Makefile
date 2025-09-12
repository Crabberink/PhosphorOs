BUILD_DIR?=build/
ASM?=gcc
CARGO_FLAGS=--target x86_64-phosphor_os.json

.PHONY: all kernel clean

all: kernel

kernel: $(BUILD_DIR)/kernel.bin

$(BUILD_DIR)/kernel.bin:
# 	$(ASM) -c main.asm -o $(BUILD_DIR)/kernel.bin
	cargo build $(CARGO_FLAGS)

clean:
	rm -f $(BUILD_DIR)/kernel.bin