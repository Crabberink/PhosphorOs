org 0x7C00
bits 16

; newline character
%define ENDL 0x0D, 0x0A

;
; FAT12 header
;
jmp short start
nop

bdb_oem: 					db 'MSWIN4.1'		; Maximum compatibility becuase microsoft
bdb_bytes_per_sector: 		dw 512
bdb_sectors_per_cluster:	db 1
bdb_reserved_sectors:		dw 1
bdb_fat_count: 				db 2
bdb_dir_entries_count: 		dw 0E0h
bdb_total_sectors: 			dw 2880				; 2880 * 512 = 1.44MB
bdb_media_descriptor_type: 	db 0F0h
bdb_sectors_per_fat: 		dw 9
bdb_sectors_per_track: 		dw 18
bdb_heads: 					dw 2
bdb_hidden_sectors: 		dd 0
bdb_large_sector_count: 	dd 0

; extended boot record
ebr_drive_number: 			db 0 				; Useless piece of crap
			 				db 0				; More uselesser
ebr_signature: 				db 29h
ebr_volume_id: 				db 'BOOB'			; Serial number, value doesn't matter
ebr_volume_label: 			db 'PHOSPHOR OS'	; 11 bytes
ebr_system_id: 				db 'FAT12   '		; 8 bytes





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
    mov ax, 0           				; can't directly write to ds/es
    mov ds, ax
    mov es, ax

    ; setup stack
    mov ss, ax
    mov sp, 0x7C00      				; stack grows downwards, and the OS partition starts at 0x7C00, so starting the stack here prevents it from overwriting program code.

	; Read something from the disk
	mov [ebr_drive_number], dl
	mov ax, 1 							; LBA = 1, second disk sector
	mov cl, 1 							; Read 1 sector
	mov bx, 0x7E00 						; End of bootloader (0x7C00) + 512 bytes
	call disk_read

    ; print message
    mov si, msg_hello
    call puts

	cli
    hlt




disk_error:
	mov si, msg_disk_error
	call puts

wait_key_and_reboot:
	mov ah, 00h
	int 16h						; wait for keypress
	jmp 0FFFFh:0				; the beginning of the BIOS, should basically reboot

.halt:
    cli							; Disable interrupts so the halt state isn't interrupted
	hlt

;
;	Disk routines
;

; Converts a logical block address to a cylinder-head-sector address
; Params:
;	- ax: logical block address
; Returns:
;	- cx [bits 0-5]: sector number
;   - cx [bits [6-15]: cylinder
;
lba_to_chs:

	push ax
	push dx

	xor dx, dx								; Clear that mf
	div word [bdb_sectors_per_track]		; ax = LBA / SectorsPerTrack
											; cx = LBA % SectorsPerTrack

	inc dx									; dx = sector = (LBA % SectorsPerTrack) + 1
	mov cx, dx								; cx = sector

	xor dx, dx								; Clear that mf again
	div word [bdb_heads]					; ax = cylinder = (LBA / SectorsPerTrack) / Heads
											; dx = head = (LBA / SectorsPerTrack) % Heads
	mov dh, dl 								; dh = head
	mov ch, al								; ch = cylinder (lower 8 bits)
	shl ah, 6
	or  cl, ah								; upper 2 bits pf cylinder in CL

	pop ax
	mov dl, al
	pop ax
	ret

; Read a sector from disk
; Params:
; 	- ax: LBA address
; 	- cl: number of sectors to read
; 	- dl: drive number
; 	- es:bx: memory address to store read data

disk_read:

	push ax
	push bx
	push cx
	push dx
	push di

	push cx									; Save CL
	call lba_to_chs							; compute CHS address
	pop ax									; AL = number of sectors (CL)
	
	mov ah, 02h
	mov di, 3

.retry:
	pusha									; Save registers because the BIOS might change them
	stc										; Set carry flag, some BIOSes require this
	int 13h									; carry flag cleared = success
	jnc .done								; If carry flag is set

	; Read failed :(
	popa									; Restore registers
	call disk_reset							; Reset disk

	dec di
	test di, di
	jnz .retry								; Retry 3 times

	;This is so sad its so over
.fail:
	jmp disk_error

.done:
	popa

	pop di
	pop dx
	pop cx
	pop bx
	pop ax
	ret


; Resets disk controller
; Params:
; 	- dl: drive number
disk_reset:
	pusha
	mov ah, 00h
	stc
	int 13h
	jc disk_error
	popa
	ret

msg_hello: 			db 'Hello world! ', ENDL, 0
msg_disk_error: 	db 'Error: Failed to read disk!', ENDL, 'Press any key to reboot...', ENDL, 0
times 510-($-$$) db 0
dw 0AA55h