use std::{
	fmt,
	iter::FusedIterator,
	ops::{Index, IndexMut},
};

#[derive(Clone, Debug)]
pub struct CsrGraph<T> {
	pub base: T,
	data: Vec<T>,
	cols: Vec<usize>,
	rows: Vec<usize>,
}
pub struct CsrIter<'a, T>(usize, &'a CsrGraph<T>);
pub struct ColIter<'a, T>(usize, usize, &'a CsrGraph<T>);
pub struct RowIter<'a, T>(usize, usize, &'a CsrGraph<T>);

impl<T> CsrGraph<T> {
	pub fn new(base: T) -> Self {
		CsrGraph {
			base,
			data: Vec::new(),
			cols: Vec::new(),
			rows: vec![0],
		}
	}

	pub fn size(&self) -> usize {
		// Rows will always have at least one entry to represent size.
		*self.rows.last().unwrap()
	}

	/// Insert `data` to the given position, returning false if it already
	///  exists.
	pub fn insert(&mut self, data: T, pos: (usize, usize)) -> bool {
		self.insert_idx(data, pos).is_ok()
	}

	/// Equivalent to `self[pos]`.
	pub fn get(&self, pos: (usize, usize)) -> &T {
		self.index(pos)
	}
	/// Equivalent to `self[pos]`, except when the `pos` entry does not exist.
	pub fn get_mut(&mut self, pos: (usize, usize)) -> Option<&mut T> {
		self.get_data_idx(pos).map(|i| &mut self.data[i])
	}

	/// Return an iterator over all entries.
	pub fn iter(&self) -> CsrIter<T> {
		CsrIter(0, &self)
	}

	/// Return an iterator over the given row.
	pub fn row_iter(&self, row: usize) -> RowIter<T> {
		RowIter(row, 0, self)
	}

	/// Return an iterator over the given column.
	pub fn col_iter(&self, col: usize) {}

	/// Insert `data` at `pos` and return the array index of the new entry,
	///  else return the existing entry index as [Err].
	fn insert_idx(&mut self, data: T, pos: (usize, usize)) -> Result<usize, usize> {
		// If desired row beyond current capacity
		if (pos.0 + 1) >= self.rows.len() {
			self.rows.resize(pos.0 + 2, *self.rows.last().unwrap());
		}

		// Try to insert value
		let row_range = self.rows[pos.0] .. self.rows[pos.0 + 1];
		match self.cols[row_range.clone()].binary_search(&pos.1) {
			// Entry exists, return false
			Ok(i) => Err(i),
			// Entry does not exist, insert
			Err(i) => {
				// Insert data at index.
				self.data.insert(row_range.start + i, data);
				// Insert column at index.
				self.cols.insert(row_range.start + i, pos.1);
				// Update rows
				for v in self.rows.iter_mut().skip(pos.0 + 1) {
					*v += 1;
				}

				Ok(i)
			}
		}
	}

	/// Get the index of the entry at `pos` if it exists.
	fn get_data_idx(&self, pos: (usize, usize)) -> Option<usize> {
		// If row beyond vector, return base. Else search row for column.
		if pos.0 >= self.rows.len() {
			None
		} else {
			// If column found within row, return data. Else return base.
			let row_range = self.rows[pos.0] .. self.rows[pos.0 + 1];
			match self.cols[row_range.clone()].binary_search(&pos.1) {
				Ok(v) => Some(row_range.start + v),
				Err(_) => None,
			}
		}
	}
}

impl<T: Default> Default for CsrGraph<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

impl<T> Index<(usize,usize)> for CsrGraph<T> {
	type Output = T;

	fn index(&self, pos: (usize, usize)) -> &Self::Output {
		match self.get_data_idx(pos) {
			Some(i) => &self.data[i],
			None => &self.base,
		}
	}
}
impl<T: Default> IndexMut<(usize, usize)> for CsrGraph<T> {
	/// Returns a mutable reference for data at `pos`, creating the entry if it
	///  does not exist.
	fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
		match self.insert_idx(T::default(), pos) {
			Ok(i) | Err(i) => &mut self.data[i],
		}
	}
}
impl<T: fmt::Display> fmt::Display for CsrGraph<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("[\n")?;

		for (pos, v) in self.iter() {
			f.write_fmt(format_args!("\t({}, {}) - {v}\n", pos.0, pos.1))?;
		}

		f.write_str("]\n")
	}
}

impl<'a, T> Iterator for CsrIter<'a, T> {
	type Item = ((usize, usize), &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		if self.0 < self.1.data.len() {
			let data = &self.1.data[self.0];
			let col = self.1.cols[self.0];
			let row = match self.1.rows.binary_search(&self.0) {
				// If idx exists in rows, it may be present in multiple entries
				//  in the case of gaps in rows. Solution is iterate till row
				//  is not empty.
				Ok(mut v) => {
					// Go forward till row is not empty
					while (self.1.rows[v + 1] - self.1.rows[v]) == 0 {
						v += 1;
					}
					v
				}
				// If idx not in rows, guaranteed to be one after row start.
				Err(v) => v-1,
			};

			self.0 += 1;

			Some(((row, col), data))
		} else {
			None
		}
	}

	fn count(self) -> usize {
		self.1.data.len() - self.0
	}
	fn last(mut self) -> Option<Self::Item> {
		self.0 = self.1.data.len() - 1;
		self.next()
	}
	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if self.0 < self.1.data.len() {
			self.0 += n;
		}
		self.next()
	}
	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.1.data.len() - self.0;
		(len, Some(len))
	}
}
impl<'a, T> ExactSizeIterator for CsrIter<'a, T> {}
impl<'a, T> FusedIterator for CsrIter<'a, T> {}

impl<'a, T> Iterator for RowIter<'a, T> {
	type Item = ((usize, usize), &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		let row_len = self.2.rows[self.0 + 1] - self.2.rows[self.0 + 1];
		if self.1 < row_len {
			let data = &self.2.data[self.0];
			let col = self.2.cols[self.0];
			let row = self.0;

			self.1 += 1;

			Some(((row, col), data))
		} else {
			None
		}
	}

	fn count(self) -> usize {
		let row_len = self.2.rows[self.0 + 1] - self.2.rows[self.0 + 1];
		row_len - self.1
	}
	fn last(mut self) -> Option<Self::Item> {
		let row_len = self.2.rows[self.0 + 1] - self.2.rows[self.0 + 1];
		self.0 = row_len - 1;
		self.next()
	}
	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if self.1 < (self.2.rows.len() - 2) {
			self.1 += n;
		}
		self.next()
	}
	fn size_hint(&self) -> (usize, Option<usize>) {
		let row_len = self.2.rows[self.0 + 1] - self.2.rows[self.0 + 1];
		let len = row_len - self.1;
		(len, Some(len))
	}
}