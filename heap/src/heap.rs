pub struct MaxHeap<T> {
	data: Vec<T>
}

impl<T: std::cmp::PartialOrd> MaxHeap<T> {
	pub fn new(n: usize) -> Self {
		MaxHeap {
			data: Vec::with_capacity(n)
		}
	}

	pub fn push(&mut self, dat: T) -> Result<(), &'static str> {
		// Since vector is used, this could be removed entirely
		if self.data.len() == self.data.capacity() {
			return Err("Heap is full.");
		}

		self.data.push(dat);

		/* Swap data with parent until heap condition satisfied, or is root
		"trickle/bubble-up"
		*/
		let mut idx = self.data.len() - 1;
		while idx > 0 {
			let parent_idx = (idx + 1) / 2 - 1;

			if self.data[parent_idx] < self.data[idx] {
				self.data.swap(parent_idx, idx);
			} else {
				break;
			}

			idx = parent_idx; // Move up
		}

		Ok(())
	}

	pub fn pop(&mut self) -> Result<T, &'static str> {
		if self.data.len() == 0 {
			return Err("Heap is empty.");
		}

		let item = self.data.swap_remove(0);

		/* Trickle-down logic
		*/
		let mut idx = 0;
		while idx < (self.data.len() / 2) {
			let l_child = 2*idx + 1;
			let r_child = l_child + 1;

			// Pick the larger child (max-heap), or the left child if right doesn't exist
			let child = if r_child >= self.data.len() {
				l_child
			} else if self.data[l_child] < self.data[r_child] {
				r_child
			} else {
				l_child
			};

			if self.data[idx] < self.data[child] {
				self.data.swap(idx, child);
			} else {
				break;
			}

			idx = child; // Move down
		}

		Ok(item)
	}

	pub fn top(&self) -> Option<&T> {
		self.data.get(0)
	}

	pub fn clear(&mut self) {
		self.data.clear();
	}

	pub fn dump(&self) -> &[T] {
		&self.data
	}
}
