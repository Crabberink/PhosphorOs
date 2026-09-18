#qemu-system-i386 -fda build/main_floppy.img -nographic
qemu-system-x86_64 -drive format=raw,file=./src/kernel/target/x86_64-phosphor_os/debug/bootimage-kernel.bin -nographic