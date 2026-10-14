use core::task::Poll;

use conquer_once::spin::OnceCell;
use crossbeam::queue::{ArrayQueue};
use futures::{Stream, StreamExt, task::AtomicWaker};
use pc_keyboard::*;

use crate::{WRITER, cprintln, task::executor, writer::WriterColor};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const UNICODE_BACKSPACE: char = '\u{8}';

pub struct ScancodeStream {
	_private: (),
}

impl ScancodeStream {
	pub fn new() -> Self {
		SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
			.expect("Scancode stream should only be called once");
		ScancodeStream { _private: () }
	}
}

impl Stream for ScancodeStream {
	type Item = u8;

	fn poll_next(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Option<Self::Item>> {
		let queue = SCANCODE_QUEUE.try_get().expect("not initialized");
		
		if let Some(scancode) = queue.pop() {
			return Poll::Ready(Some(scancode));
		}

		WAKER.register(&cx.waker());
		match queue.pop() {
			Some(scancode) => {
				WAKER.take();
				Poll::Ready(Some(scancode))
			},
			None => Poll::Pending,
		}
	}
}

pub(crate) fn add_scancode(scancode: u8) {
	let Ok(queue) = SCANCODE_QUEUE.try_get() else {
		cprintln!(WriterColor::Yellow, "WARNING: Scancode queue uninitialized");
		return;
	};

	if queue.push(scancode).is_err() {
		cprintln!(WriterColor::Yellow, "WARNING: Scancode queue full; Dropping keyboard input");
		return;
	}

	WAKER.wake();
}

pub async fn process_keypresses() {
	let mut scancodes = ScancodeStream::new();
	let mut keyboard = PS2Keyboard::new(
		ScancodeSet1::new(),
		layouts::Us104Key,
		HandleControl::Ignore
	);

	while let Some(scancode) = scancodes.next().await {
		let Ok(Some(key_event)) = keyboard.add_byte(scancode) else {
			continue;
		};
		let Some(key) = keyboard.process_keyevent(key_event) else {
			continue;
		};

		let mut console = WRITER.lock();

		match key {
			DecodedKey::Unicode(UNICODE_BACKSPACE) => {
				console.backspace();
			},
			DecodedKey::Unicode(character) => {
				match character {
					'\n' => {
						executor::spawn(print_fart());
					},
					char => {
						console.print_char(char);
					}
				}
			},
			// Keys that aren't unicode characters, like F1, Enter, etc. can be handled here
			DecodedKey::RawKey(key) => {
				match key {
					KeyCode::Return => {},
					_ => {}
				}
			},
		}
	}
}

async fn print_fart() {
	let fart = async_fart().await;

	cprintln!(WriterColor::Green, "{fart}");
}

async fn async_fart() -> &'static str {
	"fart"
}