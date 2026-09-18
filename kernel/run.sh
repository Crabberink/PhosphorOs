if [[ $* == *--vga* ]]
then
read -p "Running in -display curses"
qemu-system-x86_64 -drive format=raw,file=target/x86_64-phosphor_os/debug/bootimage-kernel.bin -display curses
else
qemu-system-x86_64 -drive format=raw,file=target/x86_64-phosphor_os/debug/bootimage-kernel.bin -nographic
fi