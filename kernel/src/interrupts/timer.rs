use crate::{cprintln, writer::WriterColor};
use x86_64::instructions::{interrupts::without_interrupts, port::{Port, PortWriteOnly}};

/// Bits 7 and 6 of the Mode/Control Register
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PITChannel {
    Channel0 = 0 << 6,
    Channel1 = 1 << 6,
    Channel2 = 2 << 6,
}

/// Bits 5 and 4 of the Mode/Control Register
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PITAccessMode {
    LatchCountValue = 0 << 4,
    LowByte = 1 << 4,
    HighByte = 2 << 4,
    LowHigh = 3 << 4,
}

// 0 0 0 = Mode 0 (interrupt on terminal count)
// 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
// 0 1 0 = Mode 2 (rate generator)
// 0 1 1 = Mode 3 (square wave generator)
// 1 0 0 = Mode 4 (software triggered strobe)
// 1 0 1 = Mode 5 (hardware triggered strobe)
// 1 1 0 = Mode 2 (rate generator, same as 010b)
// 1 1 1 = Mode 3 (square wave generator, same as 011b)
/// Bits 1-3 of the Mode/Control Register
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PITOperatingMode {
    InterruptOnTerminalCount = 0 << 1,
    HardwareRetriggerableOneShot = 1 << 1,
    RateGenerator = 2 << 1,
    SquareWaveGenerator = 3 << 1,
    SoftwareTriggeredStrobe = 4 << 1,
    HardwareTriggeredStrobe = 5 << 1,
    // Rate Generator Equivalent
    // Square Wave Generator Equivalent
}

/// Bit 0 of the Mode/Control Register
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PITCounterRepresentationMode {
    /// 16 Bit Binary Representation
    BinaryMode = 0 << 0, 
    /// 4 Digit BCD Representation
    BCDMode = 1 << 0,
}

struct PitConfiguration {
    channel: PITChannel,
    access_mode: PITAccessMode,
    operating_mode: PITOperatingMode,
    counter_representation: PITCounterRepresentationMode,
}

impl PitConfiguration {
    fn mode_command(&self) -> u8 {
        self.channel as u8 | self.access_mode as u8 | self.operating_mode as u8 | self.counter_representation as u8
    }
}

const PIT_CHANNEL0: u16 = 0x40;
const PIT_MODE_COMMAND: u16 = 0x43;

pub const PIT_BASE_HZ: u32 = 1_193_182;
// pub const PIT_BASE_HZ: u32 = 1_193_182;

pub fn setup_pit() {
    cprintln!(WriterColor::White, "Setting up pit...");

    let mut pit_mode_command: PortWriteOnly<u8> = PortWriteOnly::new(PIT_MODE_COMMAND);

    let mode_command = PitConfiguration {
        channel: PITChannel::Channel0,
        access_mode: PITAccessMode::LowHigh,
        operating_mode: PITOperatingMode::RateGenerator,
        counter_representation: PITCounterRepresentationMode::BinaryMode,
    }.mode_command();

    unsafe { pit_mode_command.write(mode_command); }
}

/// PIT must be properly setup
pub unsafe fn set_pit_divisor(divisor: u16) {
    let mut pit_channel0: Port<u8> = Port::new(PIT_CHANNEL0);
    without_interrupts(|| {
        // Low byte
        unsafe { pit_channel0.write(divisor as u8); }
        // High byte
        unsafe { pit_channel0.write((divisor >> 8) as u8);}
    });
}

/// PIT must be properly setup
pub unsafe fn set_pit_freq(hz: u32) {
    let closest_divisor = closest_pit_divisor_for(hz);

    unsafe { set_pit_divisor(closest_divisor) }
}

// freq = PIT_FREQ / divisor
// divisor * freq = PIT_FREq
// divisor = PIT_FREQ MHz / freq; Snap to 1..65536
pub fn closest_pit_divisor_for(hz: u32) -> u16 {
    let divisor = (PIT_BASE_HZ + hz/2)/hz; // Works out to (base/hz + 0.5) while remaining integer

    if divisor > 0xFFFF {
        return 0;
    }
    divisor.max(2) as u16
}