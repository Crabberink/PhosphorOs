.set ALIGN, 1<<0            # 0x00
.set MEMINFO, 1<<1          # 0x10
.set FLAGS, ALIGN | MEMINFO # 0x10+0x00 = 0x10
.set MAGIC, 0x1BADB002      # What the fuck.
                            # Someone decided that would be the start of multiboot header 
.set CHECKSUM, -(MAGIC + FLAGS)  # I guess just to prove this is a kernel and verify that this is valid

.section multiboot
.align 4        # this is the actual header
.long MAGIC     # offset 0, MAGIC number (the actual fucking term)
.long FLAGS     # offset 4, the flags 
.long CHECKSUM  # offset 8, the checksum. This thing evaluates to zero when added to magic or flags

.section .bss   # Setting up the SP
.align 16       # 16 bytes
stack_bottom:   # Stack grows back to here
.skip 16384     # 16 KB
stack_top:

.section text
.global _start
.type _start, @function
_start:
mov $stack_top, %esp    # Holy fuck I forgot GAS uses AT&T syntax

# ; Do bullshit
# gdt_start:
# dq 0x0000_0000_0000_0000    ; Null
# dq 0x00_C_F_9A_000000_FFFF  ; Kernel Code
# dq 0x00_C_F_92_000000_FFFF  ; Kernel Data
# dq 0x00_C_F_FA_000000_FFFF  ; User Code
# dq 0x00_C_F_F2_000000_FFFF  ; User Data

# gtd_end:

# gtdr:

call kernel_main

# Literally should not happen but if the kernel returns then loop forever

cli
haltLoop:
hlt
jmp haltLoop

# Set _start symbol size to current location '.' minus the start
.size _start, . - _start
