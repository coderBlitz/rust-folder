use std::{cmp::PartialOrd, mem};

#[derive(Debug)]
struct Splay<T> {
	size: usize,
	root: Option<Box<SplayNode<T>>>,
}

#[derive(Clone, Debug)]
struct SplayNode<T> {
	data: T,
	left: Option<Box<SplayNode<T>>>,
	right: Option<Box<SplayNode<T>>>,
}
impl<T> SplayNode<T> {
	fn new(data: T) -> Self {
		SplayNode {
			data,
			left: None,
			right: None,
		}
	}
}

impl<T> Splay<T> {
	/// Left rotation, assuming right child exists.
	///
	/// If a tree starts like:
	///
	/// ```ignore
	///    D
	///   / \
	///  B   F
	/// / \ / \
	/// A C E G
	/// ```
	///
	/// Then `rotate_left(D)` will result in:
	///
	/// ```ignore
	///      F
	///     / \
	///    D   G
	///   / \
	///  B   E
	/// / \
	/// A C
	/// ```
	fn rotate_left(child: &mut Box<SplayNode<T>>) {
		debug_assert!(child.right.is_some());

		// From above diagram, assume child = D.
		// r = Some(F), D_r = None
		let mut r = child.right.take();
		// r = Some(D), child = F
		mem::swap(child, r.as_mut().unwrap());

		// F_l = Some(D), r = Some(E)
		mem::swap(&mut r, &mut child.left);
		// D_r = r
		child.left.as_mut().unwrap().right = r;
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
		//let mut
	}
}

#[cfg(test)]
mod test {
	use super::*;

	/// Returns the root node for the tree below:
	/// ```
	///  A
	/// ```
	fn tree_1() -> SplayNode<char> {
		// Root node
		SplayNode::new('a')
	}

	/// Returns the root node for the tree below:
	/// ```
	///  B
	/// / \
	/// A C
	/// ```
	fn tree_2() -> SplayNode<char> {
		// Leaf nodes
		let a = SplayNode::new('a');
		let c = SplayNode::new('c');

		// Root node
		SplayNode {
			data: 'b',
			left: Some(Box::new(a)),
			right: Some(Box::new(c)),
		}
	}

	/// Returns the root node for the tree below:
	/// ```
	///    D
	///   / \
	///  B   F
	/// / \ / \
	/// A C E G
	/// ```
	fn tree_3() -> SplayNode<char> {
		// Leaf nodes
		let a = SplayNode::new('a');
		let c = SplayNode::new('c');
		let e = SplayNode::new('e');
		let g = SplayNode::new('g');

		// Middle nodes
		let b = SplayNode {
			data: 'b',
			left: Some(Box::new(a)),
			right: Some(Box::new(c)),
		};
		let f = SplayNode {
			data: 'f',
			left: Some(Box::new(e)),
			right: Some(Box::new(g)),
		};

		// Root
		SplayNode {
			data: 'd',
			left: Some(Box::new(b)),
			right: Some(Box::new(f)),
		}
	}

	/*** TESTS ***/
	/// Attempts to perform a left rotation on a single node, which is invalid.
	#[test]
	#[should_panic]
	fn rotate_left_1() {
		let root = &mut Box::new(tree_1());

		Splay::rotate_left(root);
	}

	/// Perform a left rotation on a complete h=2 tree.
	#[test]
	fn rotate_left_2() {
		let root = &mut Box::new(tree_2());

		Splay::rotate_left(root);

		assert_eq!(root.data, 'c');
		assert_eq!(root.left.as_ref().unwrap().data, 'b');
		assert!(root.right.is_none());
		assert_eq!(root.left.as_ref().unwrap().left.as_ref().unwrap().data, 'a');
	}

	/// Perform a left rotation on a complete h=3 tree.
	#[test]
	fn rotate_left_3() {
		let root = &mut Box::new(tree_3());

		Splay::rotate_left(root);

		assert_eq!(root.data, 'f');
		assert_eq!(root.left.as_ref().unwrap().data, 'd');
		assert_eq!(root.right.as_ref().unwrap().data, 'g');
		assert_eq!(
			root.left.as_ref().unwrap().right.as_ref().unwrap().data,
			'e'
		);
	}
}
