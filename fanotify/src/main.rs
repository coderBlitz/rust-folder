use std::default::Default;
use fanotify::{sys, Fanotify, InitFlags, EventFdFlags};
use std::path::Path;

fn main() {
	// Initialize the Fanotify instance
	let flags = InitFlags {
		cloexec: true,
		..InitFlags::default()
	};
	let evt_flags = EventFdFlags {
		rdonly: true,
		..EventFdFlags::default()
	};
	let fan = Fanotify::init(flags, evt_flags);

	if let Err(ref e) = fan {
		eprintln!("Fanotify init failed: {e}");
		return;
	}
	let mut fan = fan.unwrap();

	// Mark test file
	let path = Path::new("/tmp/test.txt");
	if let Err(e) = fan.add_mark(path, 0, sys::FAN_OPEN | sys::FAN_ACCESS | sys::FAN_MODIFY) {
		eprintln!("Marking '{}' failed: {e}", path.display());
		return;
	}

	// Loop until desired event(s) occurs
	let mut cnt = 0;
	while cnt < 5 {
		for e in fan.iter() {
			println!("Encountered event from PID {} with mask 0x{:X}", e.pid, e.mask);
			cnt += 1;
		}
	}
}
