org 0x7C00
bits 16

; newline character
%define ENDL 0x0D, 0x0A

start:
    jmp main

;
; Prints a string to the screen
; Params:
;   - ds:si  points to string
;
puts: 
    ; save registers we will modify
    push si
    push ax

    ; Since assembly is just a fancy way of writing a bunch of bytes, functions aren't real and everything is in sequence (these are just labels) so the execution continues to the .loop label

.loop:
    lodsb               ; loads next character in 
    or al, al           ; verify if next character is null - Sets zero flag to true if result is zero
    jz .done            ; Break if its zero

    mov ah, 0x0e        ; Text mode
    mov bh, 0x0
    int 0x10            ; Bios interrupt 0x10

    jmp .loop

.done:
    pop ax
    pop si
    ret

main:
    
    ; setup data segments
    mov ax, 0           ; can't directly write to ds/es
    mov ds, ax
    mov es, ax

    ; setup stack
    mov ss, ax
    mov sp, 0x7C00      ; stack grows downwards, and the OS partition starts at 0x7C00, so starting the stack here prevents it from overwriting program code.

    ; print message
    mov si, msg_hello
    call puts

    jmp main

    hlt

.halt:
    jmp .halt

msg_hello: db 'Hello world! ', ENDL, 0


times 510-($-$$) db 0
dw 0AA55h