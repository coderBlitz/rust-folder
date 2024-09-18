use crate::PamConv;
use crate::macros::map_enum_i32;
use crate::sys;

map_enum_i32! (
	/// PAM result/return codes.
	///
	PamResult,

	Success => sys::PAM_SUCCESS,
	OpenErr => sys::PAM_OPEN_ERR,
	SymbolErr => sys::PAM_SYMBOL_ERR,
	ServiceErr => sys::PAM_SERVICE_ERR,
	SystemErr => sys::PAM_SYSTEM_ERR,
	BufErr => sys::PAM_BUF_ERR,
	PermDenied => sys::PAM_PERM_DENIED,
	AuthErr => sys::PAM_AUTH_ERR,
	CredInsufficient => sys::PAM_CRED_INSUFFICIENT,
	AuthInfoUnavail => sys::PAM_AUTHINFO_UNAVAIL,
	UserUnknown => sys::PAM_USER_UNKNOWN,
	MaxTries => sys::PAM_MAXTRIES,
	NewAuthTokReqd => sys::PAM_NEW_AUTHTOK_REQD,
	AcctExpired => sys::PAM_ACCT_EXPIRED,
	SessionErr => sys::PAM_SESSION_ERR,
	CredUnavail => sys::PAM_CRED_UNAVAIL,
	CredExpired => sys::PAM_CRED_EXPIRED,
	CredErr => sys::PAM_CRED_ERR,
	NoModuleData => sys::PAM_NO_MODULE_DATA,
	ConvErr => sys::PAM_CONV_ERR,
	AuthTokErr => sys::PAM_AUTHTOK_ERR,
	AuthTokRecoveryErr => sys::PAM_AUTHTOK_RECOVERY_ERR,
	AuthTokLockBusy => sys::PAM_AUTHTOK_LOCK_BUSY,
	AuthTokDisableAging => sys::PAM_AUTHTOK_DISABLE_AGING,
	TryAgain => sys::PAM_TRY_AGAIN,
	Ignore => sys::PAM_IGNORE,
	Abort => sys::PAM_ABORT,
	AuthTokExpired => sys::PAM_AUTHTOK_EXPIRED,
	ModuleUnknown => sys::PAM_MODULE_UNKNOWN,
	BadItem => sys::PAM_BAD_ITEM,
	ConvAgain => sys::PAM_CONV_AGAIN,
	Incomplete => sys::PAM_INCOMPLETE,
);

map_enum_i32!(
	/// Items associated with a pam transaction.
	///
	PamItemType,

	/// PAM service name, as given to `pam_start(3)`.
	Service => sys::PAM_SERVICE,
	/// The username (post-authentication) that is allowed to use a service.
	User => sys::PAM_USER,
	/// Terminal name of client application, prefixed by `/dev/` for device files.
	Tty => sys::PAM_TTY,
	/// The requesting hostname (machine from which `Ruser` is requesting service).
	Rhost => sys::PAM_RHOST,
	/// The conversation function.
	Conv => sys::PAM_CONV,
	/// Authentication token (often a password).
	AuthTok => sys::PAM_AUTHTOK,
	/// Old authentication token (such as when changing password).
	OldAuthTok => sys::PAM_OLDAUTHTOK,
	/// The requesting user name (local for local requester, remote for remote).
	Ruser => sys::PAM_RUSER,
	/// The string used when prompting for a user's name. Defaults to localized "login: ".
	UserPrompt => sys::PAM_USER_PROMPT,

	// Linux-PAM extensions
	/// Function pointer to redirect centrally managed failure delays.
	FailDelay => sys::PAM_FAIL_DELAY,
	XDisplay => sys::PAM_XDISPLAY,
	XAuthData => sys::PAM_XAUTHDATA,
	AuthTokType => sys::PAM_AUTHTOK_TYPE,
);
impl From<PamItem> for PamItemType {
	fn from(item: PamItem) -> Self {
		match item {
			PamItem::Service(_) => PamItemType::Service,
			PamItem::User(_) => PamItemType::User,
			PamItem::UserPrompt(_) => PamItemType::UserPrompt,
			PamItem::Tty(_) => PamItemType::Tty,
			PamItem::Ruser(_) => PamItemType::Ruser,
			PamItem::Rhost(_) => PamItemType::Rhost,
			PamItem::AuthTok(_) => PamItemType::AuthTok,
			PamItem::OldAuthTok(_) => PamItemType::OldAuthTok,
			PamItem::Conv(_) => PamItemType::Conv,
		}
	}
}

map_enum_i32!(
	/// All supported conversation types.
	///
	PamConvType,

	PromptEchoOff => sys::PAM_PROMPT_ECHO_OFF,
	PromptEchoOn => sys::PAM_PROMPT_ECHO_ON,
	ErrorMsg => sys::PAM_ERROR_MSG,
	TextInfo => sys::PAM_TEXT_INFO,
	// Linux-PAM specific
	RadioType => sys::PAM_RADIO_TYPE,
	BinaryPrompt => sys::PAM_BINARY_PROMPT,
);

/// A PAM item used with [pam_get_item] and [pam_set_item].
///
pub enum PamItem {
	Service(String),
	User(String),
	UserPrompt(String),
	Tty(String),
	Ruser(String),
	Rhost(String),
	AuthTok(String),
	OldAuthTok(String),
	Conv(PamConv),
}