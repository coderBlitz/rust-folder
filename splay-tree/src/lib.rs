use std::cmp::PartialOrd;

struct Splay<T> {
	size: usize,
	root: Option<Box<SplayNode<T>>>,
}

struct SplayNode<T> {
	data: T,
	left: Option<Box<SplayNode<T>>>,
	right: Option<Box<SplayNode<T>>>,
}

impl<T> Splay<T> {
	/// Left rotation.
	///
	/// If a tree starts like:
	///
	/// ```
	///    D
	///   / \
	///  B   F
	/// / \ / \
	/// A C E G
	/// ```
	///
	/// Then `rotate_left(D)` will result in:
	///
	/// ```
	///      F
	///     / \
	///    D   G
	///   / \
	///  B   E
	/// / \
	/// A C
	/// ```
	fn rotate_left() {
	}
}

impl<T: PartialOrd> Splay<T> {
	pub fn new() -> Self {
		Splay {
			size: 0,
			root: None,
		}
	}

	fn insert(data: T) {
		let mut
	}
}
