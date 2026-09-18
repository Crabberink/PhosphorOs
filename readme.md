PhosphorOS
==========

What is this?
-------------

PhosphorOS is a basic operating system I created to learn more about the inner workings of modern operating systems and computers. It is not intended to be high quality, or useful in any way. It includes a pure 16 bit asm stage1 bootloader, a 16 bit asm/c stage2 bootloader, and the actual kernel/os.

How do I build it?
------------------

This project also includes a basic 2 stage boot loader, which of runs in 16 bit real mode, the default mode that all x86 processors start in for compatibility. Because of this, to compile the stage 2 bootloader a 16 bit C compiler is required. I personally used Open Watcom. The actual operating system is written in standard Asm,C,C++,Rust and does not need a specialized compiler.