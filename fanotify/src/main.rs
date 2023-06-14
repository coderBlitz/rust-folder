use fanotify::{sys, Fanotify};
use std::path::Path;

fn main() {
	//let flags = sys::FAN_CLASS_NOTIF | sys::FAN_CLOEXEC;
	//let flags = sys::FAN_CLOEXEC | sys::FAN_NONBLOCK;
	let flags = sys::FAN_CLOEXEC;
	let evt_flags = sys::O_RDONLY;
	let fan = Fanotify::init(flags, evt_flags);

	if let Err(ref e) = fan {
		eprintln!("Fanotify init failed: {e}");
		return;
	}
	let mut fan = fan.unwrap();

	let path = Path::new("/tmp/test.txt");
	if let Err(e) = fan.add_mark(path, 0, sys::FAN_OPEN | sys::FAN_ACCESS | sys::FAN_MODIFY) {
		eprintln!("Marking '{}' failed: {e}", path.display());
		return;
	}

	// Loop until event(s) occurs
	let mut cnt = 0;
	while cnt < 5 {
		for e in fan.iter() {
			println!("Encountered event from PID {}", e.pid);
			cnt += 1;
		}
	}
}
