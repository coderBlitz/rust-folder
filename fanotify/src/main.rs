use std::default::Default;
use fanotify::{Fanotify, InitFlags, EventFdFlags, MarkFlags, EventFlags};
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
	let fan = Fanotify::init(&flags, &evt_flags);

	if let Err(ref e) = fan {
		eprintln!("Fanotify init failed: {e}");
		return;
	}
	let mut fan = fan.unwrap();

	// Mark test file
	let path = Path::new("/tmp/test.txt");
	let mark_flags = MarkFlags::default();
	let evt_flags = EventFlags {
		open: true,
		access: true,
		modify: true,
		..EventFlags::default()
	};
	if let Err(e) = fan.add_mark(path, &mark_flags, &evt_flags) {
		eprintln!("Marking '{}' failed: {e}", path.display());
		return;
	}

	// Loop until desired event(s) occurs
	let mut cnt = 0;
	while cnt < 5 {
		for e in fan.iter() {
			println!("Encountered event from PID {}: {:?}", e.pid, e.mask);
			cnt += 1;
		}
	}
}
