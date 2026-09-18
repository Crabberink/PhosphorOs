use pc_keyboard::{DecodedKey, HandleControl, PS2Keyboard, ScancodeSet1, layouts};
use spin::Mutex;

use crate::{CONSOLE};

static KEYBOARD: Mutex<PS2Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
	PS2Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
);

const UNICODE_BACKSPACE: char = '\u{8}';

pub fn key_event_handler(scancode: u8) {
	let mut keyboard = KEYBOARD.lock();

	let mut console = CONSOLE.lock();

	if let Ok(Some(key_event)) = keyboard.add_byte(scancode) && let Some(key) = keyboard.process_keyevent(key_event) {
		match key {
			DecodedKey::Unicode(UNICODE_BACKSPACE) => {
				console.backspace();
			},
			DecodedKey::Unicode(character) => {
				console.print_char(character);
			},
			// Keys that aren't unicode characters, like F1, Enter, etc. can be handled here
			DecodedKey::RawKey(_key) => {
				
			},
		}
	}
}