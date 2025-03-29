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
    ; setup data segments
    mov ax, 0           				; can't directly write to ds/es
    mov ds, ax
    mov es, ax

    ; setup stack
    mov ss, ax
    mov sp, 0x7C00      				; stack grows downwards, and the OS partition starts at 0x7C00, so starting the stack here prevents it from overwriting program code.

	; some BIOSes might start us at 07CO:0000 rather than 0000:7C00, make sure we are in the right location
	push es
	push word .after
	retf
.after:

	; Read something from the disk
	mov [ebr_drive_number], dl

    ; show loading message
    mov si, msg_loading
    call puts

	; Retrieve drive parameters (sectors per head and head count) instead of rely on data formatted disk
	push es
	mov ah, 08h 	; 13 ah 08 Get drive parameters - 
	int 13h			
	jc disk_error
	pop es

	and cl, 0x3F
	xor ch, ch
	mov [bdb_sectors_per_track], cx		; write in the sector count

	inc dh
	mov [bdb_heads], dh					; write in the head count

	; LBA of root directory = reserved + fats * sectors_per_fat
	; can be hardcoded
	mov ax, [bdb_sectors_per_fat]		
	mov bl, [bdb_fat_count]
	xor bh, bh							; Get rid of the last 8 bits of bx
	mul bx							 	; ax = fats * sectors_per_fat
	add ax, [bdb_reserved_sectors]		; ax = LBA of root
	push ax

	; Size of root directory = (32 * number_of_entries) / bytes_per_sector
	mov ax, [bdb_sectors_per_fat]
	shl ax, 5							; nerd way to multiply ax by 32
	xor dx, dx							
	div word [bdb_bytes_per_sector]

	; if dx != 0 add 1
	test dx, dx
	jz .root_dir_after
	inc ax								; remainder != 0 so add 1
										; means the sector is only partially filled
.root_dir_after:

	; Read root dir
	mov cl, al							; cl = number of sectors to read = size of root directory
	pop ax								; ax = the earlier stashed LBA of the root
	mov dl, [ebr_drive_number]			; dl = drive number (previously saved)
	mov bx, buffer						; es:bx = buffer
	call disk_read

	; FIND THE STAGE2 (PLEASE I WANT FREEDOM FROM THIS 512 BIT HELL)
	xor bx, bx
	mov di, buffer

.find_stage2:
	mov si, file_stage2_bin
	mov cx, 11							; compare up to 11 characters
	push di
	; repe - Repeat while equal: repeats an instruction while zero flag = 1, or until cx reaches 0, with cx being decremented each time. 
	; cmpsb - compare 2 bytes at ds:si and es:di. si and di are inc when direction flag = 0 and dec when direction flag = 1
	repe cmpsb							; 
	pop di
	je .found_stage2

	add di, 32
	inc bx
	cmp bx, [bdb_dir_entries_count] ; Check if bx is still within the entries
	jl .find_stage2

	; The stage2 was never found. This is truly tragic. 
	jmp stage2_not_found_error

.found_stage2:

	; di should have the address to the entry
	mov ax, [di + 26]					; first logical cluster field (offset 26)
	mov [stage2_cluster], ax

	; load FAT from disk to memory
	mov ax, [bdb_reserved_sectors]
	mov bx, buffer
	mov cl, [bdb_sectors_per_fat]
	mov dl,	[ebr_drive_number]
	call disk_read

	; read stage2 and process FAT chain
	mov bx, STAGE2_LOAD_SEGMENT ; The buffer's segment
	mov es, bx					; Switch to that segment
	mov bx, STAGE2_LOAD_OFFSET

.load_stage2_loop:

	; Read next cluster
	mov ax, [stage2_cluster]

	; SUPER IMPORTANT: THIS IS A HARDCODED OFFSET THAT *ONLY WORKS FOR FLOPPY DISKS* IT WILL NEED TO BE CHANGEDs
	add ax, 31							; first cluster = (stage2_cluster - 2) * sectors_per_cluster + start_sector
										; start sector = reserved + fats + root dir size = 1 + 18 + 134 = 33

	mov cl, 1
	mov dl, [ebr_drive_number]
	call disk_read

	; ALSO SUPER IMPORTANT: THIS WILL OVERFLOW IF THE STAGE2.BIN IS TOO LARGE AND THE STAGE2 WILL BE F***ED
	add bx, [bdb_bytes_per_sector]

	; compute location of next cluster
	mov ax, [stage2_cluster]
	mov cx, 3
	mul cx
	mov cx, 2
	div cx								; ax = index of entry in FAT, dx = cluster mod 2

	mov si, buffer						; source register = buffer beginning
	add si, ax							; source register = fat entry
	mov ax, [ds:si]						; get the FAT table

	or dx, dx
	jz .even

.odd:
	shr ax, 4
	jmp .next_cluster_after

.even:
	and ax, 0x0FFF

.next_cluster_after:

	cmp ax, 0x0FF8						; end of chain
	jae	.read_finish					; "Jump if above or equal"?? AKA JGE but for UNSIGNED NUMBERS

	mov [stage2_cluster], ax
	jmp .load_stage2_loop

.read_finish:
	; jump to the stage2
	mov dl, [ebr_drive_number]			; dl = the boot device

	mov ax, STAGE2_LOAD_SEGMENT			; segment bullcrap
	mov ds, ax							
	mov es, ax

	jmp STAGE2_LOAD_SEGMENT:STAGE2_LOAD_OFFSET

	; THIS SHOULD NOT HAPPEN BUT YOU NEVER KNOW
	jmp wait_key_and_reboot

	cli									; Disable interrupts so the HALT state is not stopped
    hlt

disk_error:
	mov si, msg_disk_error
	call puts
	jmp wait_key_and_reboot

stage2_not_found_error:
	mov si, msg_stage2_not_found
	call puts
	jmp wait_key_and_reboot

wait_key_and_reboot:
	mov ah, 00h
	int 16h						; wait for keypress
	jmp 0FFFFh:0				; the beginning of the BIOS, should basically reboot

.halt:
    cli							; Disable interrupts so the halt state isn't interrupted
	hlt

;
; Prints a string to the screen
; Params:
;   - ds:si  points to string
;
puts: 
    ; save registers we will modify
    push si
    push ax
	push bx

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
	pop bx
    pop ax
    pop si
    ret

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
 
msg_loading: 			db 'Loading...', ENDL, 0
msg_disk_error: 		db 'ERR:Failed to read disk!', ENDL, 0
msg_stage2_not_found: 	db 'ERR:STAGE2.BIN not found!', ENDL, 0
file_stage2_bin: 		db 'STAGE2  BIN'
stage2_cluster:			dw 0 ; Pretty sure this shouldn't do anything 

STAGE2_LOAD_SEGMENT: 	equ 0x2000
STAGE2_LOAD_OFFSET: 	equ 0

times 510-($-$$) db 0
dw 0AA55h
buffer: