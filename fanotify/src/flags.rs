use crate::sys;

/// Source: fanotify_init
#[derive(Clone, Copy, Debug, Default)]
pub struct InitFlags {
	/// Mutually exclusive with content and notif
	pub pre_content: bool,
	/// Mutually exclusive with pre-content and notif
	pub content: bool,
	/// Mutually exclusive with pre-content and content
	pub notif: bool,
	pub cloexec: bool,
	pub nonblock: bool,
	/// Requires CAP_SYS_ADMIN
	pub unlimited_queue: bool,
	/// Requires CAP_SYS_ADMIN
	pub unlimited_marks: bool,
	/// Requires CAP_SYS_ADMIN
	pub report_tid: bool,
	/// Requires CAP_AUDIT_WRITE
	pub enable_audit: bool,
	/// Mutually exclusive with pre-content and content
	pub report_fid: bool,
	pub report_dir_fid: bool,
	// Must be provided with report_dir_fid, so use report_dfid_name.
	//pub report_name: bool,
	/// Synonym for (report_dir_fid | report_name)
	pub report_dfid_name: bool,
	// Must be provided in conjunction with report_fid and report_dfid_name, so use report_dfid_name_target.
	//pub report_target_fid: bool,
	/// Synonym for (report_dfid_name | report_fid | report_target_fid)
	pub report_dfid_name_target: bool,
	/// Mutually exclusive with report_tid
	pub report_pidfd: bool
}
impl InitFlags {
	/// Convert the struct values to the integer representation. TODO: Rename?
	///
	/// TODO: Make macro if possible
	pub fn to_bits(&self) -> i32 {
		let mut flags = 0;
		flags |= if self.pre_content { sys::FAN_CLASS_PRE_CONTENT } else { 0 };
		flags |= if self.content { sys::FAN_CLASS_CONTENT } else { 0 };
		flags |= if self.notif { sys::FAN_CLASS_NOTIF } else { 0 };
		flags |= if self.cloexec { sys::FAN_CLOEXEC } else { 0 };
		flags |= if self.nonblock { sys::FAN_NONBLOCK } else { 0 };
		flags |= if self.unlimited_queue { sys::FAN_UNLIMITED_QUEUE } else { 0 };
		flags |= if self.unlimited_marks { sys::FAN_UNLIMITED_MARKS } else { 0 };
		flags |= if self.report_tid { sys::FAN_REPORT_TID } else { 0 };
		flags |= if self.enable_audit { sys::FAN_ENABLE_AUDIT } else { 0 };
		flags |= if self.report_fid { sys::FAN_REPORT_FID } else { 0 };
		flags |= if self.report_dir_fid { sys::FAN_REPORT_DIR_FID } else { 0 };
		//flags |= if self.report_name { sys::FAN_REPORT_NAME } else { 0 };
		flags |= if self.report_dfid_name { sys::FAN_REPORT_DFID_NAME } else { 0 };
		//flags |= if self.report_target_fid { sys::FAN_REPORT_TARGET_FID } else { 0 };
		flags |= if self.report_dfid_name_target { sys::FAN_REPORT_DFID_NAME_TARGET } else { 0 };
		flags |= if self.report_pidfd { sys::FAN_REPORT_PIDFD } else { 0 };
		flags
	}
}

/// Source: fanotify_init
#[derive(Clone, Copy, Debug, Default)]
pub struct EventFdFlags {
	pub rdonly: bool,
	pub wronly: bool,
	pub rdwr: bool,
	pub largefile: bool,
	pub cloexec: bool,
	pub append: bool,
	pub dsync: bool,
	pub noatime: bool,
	pub nonblock: bool,
	pub sync: bool
}
impl EventFdFlags {
	pub fn to_bits(&self) -> i32 {
		let mut flags = 0;
		flags |= if self.rdonly { sys::O_RDONLY } else { 0 };
		flags |= if self.wronly { sys::O_WRONLY } else { 0 };
		flags |= if self.rdwr { sys::O_RDWR } else { 0 };
		flags |= if self.largefile { sys::O_LARGEFILE } else { 0 };
		flags |= if self.cloexec { sys::O_CLOEXEC } else { 0 };
		flags |= if self.append { sys::O_APPEND } else { 0 };
		flags |= if self.dsync { sys::O_DSYNC } else { 0 };
		flags |= if self.noatime { sys::O_NOATIME } else { 0 };
		flags |= if self.nonblock { sys::O_NONBLOCK } else { 0 };
		flags |= if self.sync { sys::O_SYNC } else { 0 };
		flags
	}
}

/// Source: fanotify_mark
// TODO: Convert inode/mount/filesystem to an enum, defaulting to inode
// Types are mutually exclusive, see [https://elixir.bootlin.com/linux/v6.3.8/source/fs/notify/fanotify/fanotify_user.c#L1660]
#[derive(Clone, Copy, Debug, Default)]
pub struct MarkFlags {
	pub dont_follow: bool,
	pub onlydir: bool,
	pub mount: bool,
	pub filesystem: bool,
	pub ignored_mask: bool,
	pub ignore: bool,
	pub ignored_surv_modify: bool,
	pub ignore_surv: bool,
	pub evictable: bool
}
impl MarkFlags {
	pub fn to_bits(&self) -> i32 {
		let mut flags = 0;
		flags |= if self.dont_follow { sys::FAN_MARK_DONT_FOLLOW } else { 0 };
		flags |= if self.onlydir { sys::FAN_MARK_ONLYDIR } else { 0 };
		flags |= if self.mount { sys::FAN_MARK_MOUNT } else { 0 };
		flags |= if self.filesystem { sys::FAN_MARK_FILESYSTEM } else { 0 };
		flags |= if self.ignored_mask { sys::FAN_MARK_IGNORED_MASK } else { 0 };
		flags |= if self.ignore { sys::FAN_MARK_IGNORE } else { 0 };
		flags |= if self.ignored_surv_modify { sys::FAN_MARK_IGNORED_SURV_MODIFY } else { 0 };
		flags |= if self.ignore_surv { sys::FAN_MARK_IGNORE_SURV } else { 0 };
		flags |= if self.evictable { sys::FAN_MARK_EVICTABLE } else { 0 };
		flags
	}
	pub fn from_bits(flags: i32) -> Self {
		Self {
			dont_follow: (flags & sys::FAN_MARK_DONT_FOLLOW) != 0,
			onlydir: (flags & sys::FAN_MARK_ONLYDIR) != 0,
			mount: (flags & sys::FAN_MARK_MOUNT) != 0,
			filesystem: (flags & sys::FAN_MARK_FILESYSTEM) != 0,
			ignored_mask: (flags & sys::FAN_MARK_IGNORED_MASK) != 0,
			ignore: (flags & sys::FAN_MARK_IGNORE) != 0,
			ignored_surv_modify: (flags & sys::FAN_MARK_IGNORED_SURV_MODIFY) != 0,
			ignore_surv: (flags & sys::FAN_MARK_IGNORE_SURV) != 0,
			evictable: (flags & sys::FAN_MARK_EVICTABLE) != 0,
		}
	}
}

/// Source: fanotify_mark
#[derive(Clone, Copy, Debug, Default)]
pub struct EventFlags {
	pub access: bool,
	pub access_perm: bool,
	pub attrib: bool, // (since linux 5.1)
	pub close: bool, // (close_write | close_nowrite)
	pub close_write: bool,
	pub close_nowrite: bool,
	pub create: bool,
	pub delete: bool, // (since linux 5.1)
	pub delete_self: bool, // (since linux 5.1)
	/// Exclusive to mark mask
	pub event_on_child: bool,
	pub fs_error: bool, // (since linux 5.16)
	pub modify: bool,
	pub move_self: bool, // (since linux 5.1)
	pub moved: bool, // (moved_from | moved_to)
	pub moved_from: bool, // (since linux 5.1)
	pub moved_to: bool,
	pub ondir: bool,
	pub open: bool,
	pub open_exec: bool, //(since linux 5.0)
	pub open_exec_perm: bool, // (since linux 5.0)
	pub open_perm: bool,
	/// Exclusive to event mask
	pub q_overflow: bool,
	pub rename: bool, // (since linux 5.17)
}
impl EventFlags {
	pub fn to_bits(&self) -> i32 {
		let mut flags = 0;
		flags |= if self.access { sys::FAN_ACCESS } else { 0 };
		flags |= if self.access_perm { sys::FAN_ACCESS_PERM } else { 0 };
		flags |= if self.attrib { sys::FAN_ATTRIB } else { 0 };
		flags |= if self.close { sys::FAN_CLOSE } else { 0 };
		flags |= if self.close_write { sys::FAN_CLOSE_WRITE } else { 0 };
		flags |= if self.close_nowrite { sys::FAN_CLOSE_NOWRITE } else { 0 };
		flags |= if self.create { sys::FAN_CREATE } else { 0 };
		flags |= if self.delete { sys::FAN_DELETE } else { 0 };
		flags |= if self.delete_self { sys::FAN_DELETE_SELF } else { 0 };
		flags |= if self.event_on_child { sys::FAN_EVENT_ON_CHILD } else { 0 };
		flags |= if self.fs_error { sys::FAN_FS_ERROR } else { 0 };
		flags |= if self.modify { sys::FAN_MODIFY } else { 0 };
		flags |= if self.move_self { sys::FAN_MOVE_SELF } else { 0 };
		flags |= if self.moved { sys::FAN_MOVE } else { 0 };
		flags |= if self.moved_from { sys::FAN_MOVED_FROM } else { 0 };
		flags |= if self.moved_to { sys::FAN_MOVED_TO } else { 0 };
		flags |= if self.ondir { sys::FAN_ONDIR } else { 0 };
		flags |= if self.open { sys::FAN_OPEN } else { 0 };
		flags |= if self.open_exec { sys::FAN_OPEN_EXEC } else { 0 };
		flags |= if self.open_exec_perm { sys::FAN_OPEN_EXEC_PERM } else { 0 };
		flags |= if self.open_perm { sys::FAN_OPEN_PERM } else { 0 };
		flags |= if self.q_overflow { sys::FAN_Q_OVERFLOW } else { 0 };
		flags |= if self.rename { sys::FAN_RENAME } else { 0 };
		flags
	}
	pub fn from_bits(flags: i32) -> Self {
		Self {
			access: (flags & sys::FAN_ACCESS) != 0,
			access_perm: (flags & sys::FAN_ACCESS_PERM) != 0,
			attrib: (flags & sys::FAN_ATTRIB) != 0,
			close: (flags & sys::FAN_CLOSE) != 0,
			close_write: (flags & sys::FAN_CLOSE_WRITE) != 0,
			close_nowrite: (flags & sys::FAN_CLOSE_NOWRITE) != 0,
			create: (flags & sys::FAN_CREATE) != 0,
			delete: (flags & sys::FAN_DELETE) != 0,
			delete_self: (flags & sys::FAN_DELETE_SELF) != 0,
			event_on_child: (flags & sys::FAN_EVENT_ON_CHILD) != 0,
			fs_error: (flags & sys::FAN_FS_ERROR) != 0,
			modify: (flags & sys::FAN_MODIFY) != 0,
			move_self: (flags & sys::FAN_MOVE_SELF) != 0,
			moved: (flags & sys::FAN_MOVE) != 0,
			moved_from: (flags & sys::FAN_MOVED_FROM) != 0,
			moved_to: (flags & sys::FAN_MOVED_TO) != 0,
			ondir: (flags & sys::FAN_ONDIR) != 0,
			open: (flags & sys::FAN_OPEN) != 0,
			open_exec: (flags & sys::FAN_OPEN_EXEC) != 0,
			open_exec_perm: (flags & sys::FAN_OPEN_EXEC_PERM) != 0,
			open_perm: (flags & sys::FAN_OPEN_PERM) != 0,
			q_overflow: (flags & sys::FAN_Q_OVERFLOW) != 0,
			rename: (flags & sys::FAN_RENAME) != 0,
		}
	}
}
