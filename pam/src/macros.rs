// TODO: Move `pub enum bla` into user text, to only capture members of enum.
//       New call would look like map_enum_i32!( pub enum MyEnum { ... });
//       This permits user to modify the pub as desired, and makes macro look nicer overall.
macro_rules! map_enum_i32 {
	(
		$(#[$enum_attrs:meta])*
		$name:ident,
		$(
			$(#[$entry_attrs:meta])*
			$enum_name:ident => $const_name:path,
		)*
	) => {
		// Enum definition
		$(#[$enum_attrs])*
		#[derive(Clone, Copy, Debug, Eq, PartialEq)]
		pub enum $name {
			$(
				$(#[$entry_attrs])*
				$enum_name = $const_name as isize,
			)*
		}

		// TryFrom for enum
		impl TryFrom<i32> for $name {
			type Error = ();
			fn try_from(v: i32) -> Result<Self, ()> {
				use $name::*;
				match v {
					$($const_name => Ok($enum_name),)*
					_ => Err(()),
				}
			}
		}
	}
}
pub(crate) use map_enum_i32;
