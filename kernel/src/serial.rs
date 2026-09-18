use core::arch::asm;

// Written before x86_64 was added

// Internal Serial Clock 115200hz / divisor
// DLAB 0: 0 - R/W Data
// DLAB 0: 1 - Enable Interrupt Reg
// DLAB 1: 0 - LSB BAUD DIVISOR
// DLAB 1: 1 - MSB BAUD DIVISOR
pub struct SerialPort {
    address: u16,
    divisor: u16,
    valid: bool,
}

impl SerialPort {
    pub fn new(address: u16) -> SerialPort {
        let mut serial_port: SerialPort = SerialPort {
            address:address,
            divisor:3,
            valid:false,
        };
        serial_port.init();

        serial_port
    }

    fn init(&mut self) {
        unsafe {
            out_b(self.address + 1, 0x00);  // Disable Interrupts
            out_b(self.address + 3, 0x80);  // Set DLAB to 1

            out_b(self.address + 0, self.divisor.to_le_bytes()[0]);  // Divisor = 3
            out_b(self.address + 1, self.divisor.to_le_bytes()[1]);  // 38400 Baud

            let lcr = 
                (0 << 7) + // DLAB
                (0 << 6) + // Break Enable Bit
                (0 << 3) + // No Parity
                (0 << 2) + // 1 Stop Bit
                (3 << 0);  // Data Bits (3 + 5 = 8 bits)
            out_b(self.address + 3, lcr);

            out_b(self.address + 2, 0xC7);  // Enable FIFO, clear them, 14bit threshold
            out_b(self.address + 4, 0x0B);  // Modem control shit
            
            out_b(self.address + 4, 0x1E);  // Set to loopback mode
            out_b(self.address, 0xAE);  // Send 0xAE test byte

            self.valid = true;
            if in_b(self.address) != 0xAE {
                self.valid = false;
            }

            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            out_b(self.address + 4, 0x0F);
        }
    }

    // Returns true if a signal has been received (DR Flag)
    fn is_serial_received(&self) -> bool {
        unsafe {
            return (in_b(self.address + 5) & 1) == 1;
        }
    }

    fn is_transmit_empty(&self) -> bool {
        unsafe {
            return in_b(self.address + 5) & 0x20 == 0;
        }
    }

    // Halts until data is available
    pub fn read_serial(&self) -> u8 {
        if !self.valid { return 0; }

        while !self.is_serial_received() {
            // do nothing
        }

        unsafe {
            return in_b(self.address);
        }
    }

    // Halts until transmit is empty
    pub fn write_serial(&self, byte: u8) {
        if !self.valid { return; }

        while self.is_transmit_empty() {
            // Do nothing
        }

        unsafe {
            out_b(self.address, byte);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

unsafe fn out_b(port: u16, byte: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") byte,
            options(nomem, nostack, preserves_flags)
        )
    }
}

unsafe fn in_b(port: u16) -> u8{
    let val: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") val,
            options(nomem, nostack, preserves_flags)
        )
    }
    val
}