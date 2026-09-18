PhosphorOS
==========

What is this?
-------------

PhosphorOS is a basic operating system I am creating to learn more about the inner workings of modern operating systems and computers. It is not intended to be high quality, or useful in any way. It is using the rust bootloader crate to generate a bootloader for the kernel, and is based on the blogOS tutorial: [BlogOS](https://os.phil-opp.com/)

How do I build it?
-------------
```
cargo build
```
Cargo is configured to automatically build the project and run it using qemu via
```
cargo run
```
