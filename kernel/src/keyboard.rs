use pc_keyboard::*;
use spin::Mutex;

use crate::{CONSOLE, Console};

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
				match character {
					'\n' => {
						enter_test(&mut console);
					},
					char => {
						console.print_char(char);
					}
				}
			},
			// Keys that aren't unicode characters, like F1, Enter, etc. can be handled here
			DecodedKey::RawKey(key) => {
				match key {
					KeyCode::Return => enter_test(&mut console),
					_ => {}
				}
			},
		}
	}
}

fn enter_test(console: &mut Console) {
	console.print_char('\n');
	console.print_str("test\n");
}