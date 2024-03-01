use crate::macros::*;

use std::{
	ffi::CString,
	ops::BitOr,
};

map_enum_i32!(SyslogFacility,
	Auth => libc::LOG_AUTH,
	AuthPriv => libc::LOG_AUTHPRIV,
	Cron => libc::LOG_CRON,
	Daemon => libc::LOG_DAEMON,
	Ftp => libc::LOG_FTP,
	Kern => libc::LOG_KERN,
	Local0 => libc::LOG_LOCAL0,
	Local1 => libc::LOG_LOCAL1,
	Local2 => libc::LOG_LOCAL2,
	Local3 => libc::LOG_LOCAL3,
	Local4 => libc::LOG_LOCAL4,
	Local5 => libc::LOG_LOCAL5,
	Local6 => libc::LOG_LOCAL6,
	Local7 => libc::LOG_LOCAL7,
	Lpr => libc::LOG_LPR,
	Mail => libc::LOG_MAIL,
	News => libc::LOG_NEWS,
	Syslog => libc::LOG_SYSLOG,
	User => libc::LOG_USER,
	Uucp => libc::LOG_UUCP,
);

map_enum_i32!(SyslogLevel,
	Emerg => libc::LOG_EMERG,
	Alert => libc::LOG_ALERT,
	Crit => libc::LOG_CRIT,
	Error => libc::LOG_ERR,
	Warn => libc::LOG_WARNING,
	Notice => libc::LOG_NOTICE,
	Info => libc::LOG_INFO,
	Debug => libc::LOG_DEBUG,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyslogOpts(i32);
#[allow(dead_code)]
impl SyslogOpts {
	pub const CONSOLE: Self = Self(libc::LOG_CONS);
	pub const NODELAY: Self = Self(libc::LOG_NDELAY);
	pub const NOWAIT: Self = Self(libc::LOG_NOWAIT);
	pub const DELAY: Self = Self(libc::LOG_ODELAY);
	pub const PERROR: Self = Self(libc::LOG_PERROR);
	pub const PID: Self = Self(libc::LOG_PID);
}
impl BitOr for SyslogOpts {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}


/*** Functions
***/
/// Opens a connection to the syslog interface.
///
#[inline(always)]
pub fn openlog<S: AsRef<str>>(service_name: S, opts: SyslogOpts, facility: SyslogFacility) -> CString {
	fn inner(s: &str, o: SyslogOpts, f: SyslogFacility) -> CString {
		let c = CString::new(s).unwrap();
		unsafe {
			libc::openlog(c.as_ptr(), o.0, f as i32)
		}

		c
	}
	inner(service_name.as_ref(), opts, facility)
}

/// Sends a message string to the syslog interface.
///
#[inline(always)]
pub fn syslog<S: Into<Vec<u8>>>(message: S, level: SyslogLevel) {
	let s = CString::new(message).unwrap();

	unsafe { libc::syslog(level as i32, s.as_ptr()) }
}

/// Closes the syslog interface connection.
///
#[inline(always)]
pub fn closelog() {
	unsafe { libc::closelog() }
}
