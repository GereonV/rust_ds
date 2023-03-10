use std::{fmt::Debug, hash::Hash, iter::FusedIterator, marker::PhantomData, ptr::NonNull};

struct NodePtr<T> {
	ptr: Option<NonNull<Node<T>>>,
}

impl<T> NodePtr<T> {
	fn new(elt: T, prev: &NodePtr<T>, next: &NodePtr<T>) -> NodePtr<T> {
		let raw = Box::into_raw(
			Box::new(Node {
				value: elt,
				next: *next,
				prev: *prev,
			})
		);
		NodePtr {
			ptr: Some(unsafe { NonNull::new_unchecked(raw) }),
		}
	}

	fn into_box(self) -> Option<Box<Node<T>>> {
		self.ptr.map(|valid_ptr| unsafe {
			Box::from_raw(valid_ptr.as_ptr())
		})
	}

	fn into_box_unchecked(self) -> Box<Node<T>> {
		unsafe { Box::from_raw(self.ptr.unwrap_unchecked().as_ptr()) }
	}

	fn as_ref<'a>(&self) -> Option<&'a Node<T>> {
		self.ptr.map(|valid_ptr| unsafe {
			valid_ptr.as_ref()
		})
	}

	fn as_mut<'a>(&mut self) -> Option<&'a mut Node<T>> {
		self.ptr.as_mut().map(|valid_ptr| unsafe {
			valid_ptr.as_mut()
		})
	}

	fn as_mut_unchecked<'a>(&self) -> &'a mut Node<T> {
		unsafe { self.ptr.unwrap_unchecked().as_mut() }
	}
}

impl<T> Clone for NodePtr<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for NodePtr<T> {}

impl<T> Default for NodePtr<T> {
	fn default() -> Self {
		NodePtr { ptr: None }
	}
}

unsafe impl<T: Sync> Send for NodePtr<T> {}

unsafe impl<T: Sync> Sync for NodePtr<T> {}

struct Node<T> {
	value: T,
	next: NodePtr<T>,
	prev: NodePtr<T>,
}

pub struct Iter<'a, T> {
	head: NodePtr<T>,
	tail: NodePtr<T>,
	left: usize,
	phantom: PhantomData<&'a T>,
}

pub struct IterMut<'a, T> {
	head: NodePtr<T>,
	tail: NodePtr<T>,
	left: usize,
	phantom: PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
	list: LinkedList<T>,
}

pub struct Cursor<'a, T> {
	next_index: usize,
	current: NodePtr<T>,
	list: &'a LinkedList<T>,
}

pub struct CursorMut<'a, T> {
	next_index: usize,
	current: NodePtr<T>,
	list: &'a mut LinkedList<T>,
}

pub struct DrainFilter<'a, T, F: FnMut(&mut T) -> bool> {
	current: NodePtr<T>,
	pred: F,
	list: &'a mut LinkedList<T>,
}

pub struct LinkedList<T> {
	head: NodePtr<T>,
	tail: NodePtr<T>,
	len: usize,
}

impl<T> LinkedList<T> {
	pub fn new() -> LinkedList<T> {
		LinkedList {
			head: Default::default(),
			tail: Default::default(),
			len: 0,
		}
	}

	pub fn append(&mut self, other: &mut LinkedList<T>) {
		if other.is_empty() {
			return;
		} else if self.is_empty() {
			std::mem::swap(self, other);
			return;
		}
		self.len += other.len;
		self.tail.as_mut_unchecked().next = other.head;
		other.head.as_mut_unchecked().prev = std::mem::replace(&mut self.tail, other.tail);
		std::mem::forget(other);
	}

	pub fn iter(&self) -> Iter<T> {
        Iter {
			head: self.head,
			tail: self.tail,
			left: self.len(),
			phantom: PhantomData,
		}
	}

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
			head: self.head,
			tail: self.tail,
			left: self.len(),
			phantom: PhantomData,
		}
	}

	pub fn cursor_front(&self) -> Cursor<T> {
		Cursor { next_index: 1, current: self.head, list: self }
	}

	pub fn cursor_front_mut(&mut self) -> CursorMut<T> {
		CursorMut { next_index: 1, current: self.head, list: self }
	}

	pub fn cursor_back(&self) -> Cursor<T> {
		Cursor { next_index: self.len(), current: self.tail, list: self }
	}

	pub fn cursor_back_mut(&mut self) -> CursorMut<T> {
		CursorMut { next_index: self.len(), current: self.tail, list: self }
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn clear(&mut self) {
		*self = Self::new()
	}

	pub fn contains(&self, x: &T) -> bool
	where
		T: PartialEq<T>,
	{
		self.iter().any(|elt| elt == x)
	}

	pub fn front(&self) -> Option<&T> {
		self.head.as_ref().map(|node| &node.value)
	}

	pub fn front_mut(&mut self) -> Option<&mut T> {
		self.head.as_mut().map(|node| &mut node.value)
	}

	pub fn back(&self) -> Option<&T> {
		self.tail.as_ref().map(|node| &node.value)
	}

	pub fn back_mut(&mut self) -> Option<&mut T> {
		self.tail.as_mut().map(|node| &mut node.value)
	}

	pub fn push_front(&mut self, elt: T) {
		let new_head = NodePtr::new(elt, &Default::default(), &self.head);
		let mut old_head = std::mem::replace(&mut self.head, new_head);
		if let Some(old_node) = old_head.as_mut() {
			old_node.prev = new_head;
		} else {
			self.tail = new_head;
		}
		self.len += 1;
	}

	fn _pop_front(&mut self) -> Option<T> {
		let boxed_head = self.head.into_box_unchecked();
		self.head = boxed_head.next;
		self.len -= 1;
		if let Some(new_head_node) = self.head.as_mut() {
			new_head_node.prev = Default::default();
		}
		Some(boxed_head.value)
	}

	pub fn pop_front(&mut self) -> Option<T> {
		match self.len() {
			0 => None,
			_ => self._pop_front(),
		}
	}

	pub fn push_back(&mut self, elt: T) {
		let new_tail = NodePtr::new(elt, &self.tail, &Default::default());
		let mut old_tail = std::mem::replace(&mut self.tail, new_tail);
		if let Some(old_node) = old_tail.as_mut() {
			old_node.next = new_tail;
		} else {
			self.head = new_tail;
		}
		self.len += 1;
	}

	fn _pop_back(&mut self) -> Option<T> {
		let boxed_tail = self.tail.into_box_unchecked();
		self.tail = boxed_tail.prev;
		self.len -= 1;
		if let Some(new_tail_node) = self.tail.as_mut() {
			new_tail_node.next = Default::default();
		}
		Some(boxed_tail.value)
	}

	pub fn pop_back(&mut self) -> Option<T> {
		match self.len() {
			0 => None,
			_ => self._pop_back(),
		}
	}

	fn _cursor_at_mut(&mut self, at: usize) -> CursorMut<T> {
		// TODO optimize by going from back
		let mut node_ptr = self.head;
		for _ in 0..at {
			node_ptr = node_ptr.as_mut_unchecked().next;
		}
		CursorMut { next_index: at + 1, current: node_ptr, list: self }
	}

	pub fn split_off(&mut self, at: usize) -> LinkedList<T> {
		if at == 0 {
			std::mem::take(self)
		} else if at <= self.len() {
			self._cursor_at_mut(at - 1).split_after()
		} else {
			panic!("Cannot split off at a nonexistent index")
		}
	}

	pub fn remove(&mut self, at: usize) -> T {
		assert!(at < self.len(), "Cannot remove at an index outside of the list bounds");
		unsafe { self._cursor_at_mut(at).remove_current().unwrap_unchecked() }
	}

	pub fn drain_filter<F: FnMut(&mut T) -> bool>(&mut self, filter: F) -> DrainFilter<T, F> {
		DrainFilter { current: self.head, pred: filter, list: self }
	}
}

impl<T: Clone> Clone for LinkedList<T> {
	fn clone(&self) -> Self {
		self.iter().map(|elt| elt.clone()).collect()
	}
}

impl<T: Debug> Debug for LinkedList<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_list().entries(self).finish()
	}
}

impl<T> Default for LinkedList<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Drop for LinkedList<T> {
	fn drop(&mut self) {
		let mut node_ptr = self.head;
		while let Some(boxed) = node_ptr.into_box() {
			node_ptr = boxed.next;
		}
	}
}

impl<'a, T: Copy> Extend<&'a T> for LinkedList<T> {
	fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
		iter.into_iter().for_each(|elt| self.push_back(*elt));
	}
}

impl<T> Extend<T> for LinkedList<T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		iter.into_iter().for_each(|elt| self.push_back(elt));
	}
}

impl<T, const N: usize> From<[T; N]> for LinkedList<T> {
	fn from(value: [T; N]) -> Self {
		value.into_iter().collect()
	}
}

impl<T> FromIterator<T> for LinkedList<T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut list = Self::new();
		list.extend(iter);
		list
	}
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		state.write_usize(self.len()); // write_length_prefix
		for elt in self {
			elt.hash(state);
		}
	}
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
	type Item = &'a T;
	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			head: self.head,
			tail: self.tail,
			left: self.len(),
			phantom: PhantomData,
		}
	}
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
	type Item = &'a mut T;
	type IntoIter = IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			head: self.head,
			tail: self.tail,
			left: self.len(),
			phantom: PhantomData,
		}
	}
}

impl<T> IntoIterator for LinkedList<T> {
	type Item = T;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		Self::IntoIter {
			list: self
		}
	}
}

impl<T: Ord> Ord for LinkedList<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.iter().cmp(other)
	}
}

impl<T: PartialEq<T>> PartialEq<LinkedList<T>> for LinkedList<T> {
	fn eq(&self, other: &LinkedList<T>) -> bool {
		self.iter().eq(other)
	}
}

impl<T: PartialOrd<T>> PartialOrd<LinkedList<T>> for LinkedList<T> {
	fn partial_cmp(&self, other: &LinkedList<T>) -> Option<std::cmp::Ordering> {
		self.iter().partial_cmp(other)
	}
}

impl<T: Eq> Eq for LinkedList<T> {}

// Iter

impl<T> Clone for Iter<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Iter<'_, T> {}

impl<T: Debug> Debug for Iter<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Iter")
			.field(&*std::mem::ManuallyDrop::new( LinkedList { head: self.head, tail: self.tail, len : self.left, }))
			.field(&self.left)
			.finish()
	}
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
		let node = self.tail.as_ref()?;
		self.tail = node.prev;
		self.left -= 1;
		Some(&node.value)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
	fn len(&self) -> usize {
		return self.left
	}
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		let node = self.head.as_ref()?;
		self.head = node.next;
		self.left -= 1;
		Some(&node.value)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.left, Some(self.left))
	}

	fn last(mut self) -> Option<Self::Item>
	where
		Self: Sized,
	{
		self.next_back()
	}
}

// IterMut

impl<T: Debug> Debug for IterMut<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Iter")
			.field(&*std::mem::ManuallyDrop::new( LinkedList { head: self.head, tail: self.tail, len : self.left, }))
			.field(&self.left)
			.finish()
	}
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
		let node = self.tail.as_mut()?;
		self.tail = node.prev;
		self.left -= 1;
		Some(&mut node.value)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
	fn len(&self) -> usize {
		return self.left
	}
}

impl<T> FusedIterator for IterMut<'_, T> {}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		let node = self.head.as_mut()?;
		self.head = node.next;
		self.left -= 1;
		Some(&mut node.value)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.left, Some(self.left))
	}

	fn last(mut self) -> Option<Self::Item>
	where
		Self: Sized,
	{
		self.next_back()
	}
}

// IntoIter

impl<T: Debug> Debug for IntoIter<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("IntoIter").field(&self.list).finish()
	}
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
		self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
	fn len(&self) -> usize {
		self.list.len()
	}
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.list.pop_front()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.list.len(), Some(self.list.len()))
	}

	fn last(mut self) -> Option<Self::Item>
	where
		Self: Sized,
	{
		self.next_back()
	}
}

// Cursor

impl<'a, T> Cursor<'a, T> {
	pub fn index(&self) -> Option<usize> {
		self.current.as_ref()?;
		Some(self.next_index - 1)
	}

	pub fn move_next(&mut self) {
		if let Some(node) = self.current.as_ref() {
			self.current = node.next;
			self.next_index += 1;
		} else {
			self.current = self.list.head;
			self.next_index = 1;
		}
	}

	pub fn move_prev(&mut self) {
		if let Some(node) = self.current.as_ref() {
			self.current = node.prev;
			self.next_index -= 1;
		} else {
			self.current = self.list.tail;
			self.next_index = self.list.len()
		}
	}

	pub fn current(&self) -> Option<&'a T> {
		self.current.as_ref().map(|node| &node.value)
	}

	pub fn peek_next(&self) -> Option<&'a T> {
		self.current.as_ref()
			.map_or(self.list.head, |node| node.next).as_ref()
			.map(|node| &node.value)
	}

	pub fn peek_prev(&self) -> Option<&'a T> {
		self.current.as_ref()
			.map_or(self.list.tail, |node| node.prev).as_ref()
			.map(|node| &node.value)
	}

	pub fn front(&self) -> Option<&'a T> {
		self.list.head.as_ref().map(|node| &node.value)
	}

	pub fn back(&self) -> Option<&'a T> {
		self.list.tail.as_ref().map(|node| &node.value)
	}
}

impl<T> Clone for Cursor<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Cursor<'_, T> {}

impl<T: Debug> Debug for Cursor<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Cursor")
			.field(&self.list)
			.field(&self.index())
			.finish()
	}
}

// CursorMut

impl<'a, T> CursorMut<'a, T> {
	pub fn index(&self) -> Option<usize> {
		self.current.as_ref()?;
		Some(self.next_index - 1)
	}

	pub fn move_next(&mut self) {
		if let Some(node) = self.current.as_ref() {
			self.current = node.next;
			self.next_index += 1;
		} else {
			self.current = self.list.head;
			self.next_index = 1;
		}
	}

	pub fn move_prev(&mut self) {
		if let Some(node) = self.current.as_ref() {
			self.current = node.prev;
			self.next_index -= 1;
		} else {
			self.current = self.list.tail;
			self.next_index = self.list.len()
		}
	}

	pub fn current(&mut self) -> Option<&mut T> {
		self.current.as_mut().map(|node| &mut node.value)
	}

	pub fn peek_next(&mut self) -> Option<&mut T> {
		self.current.as_ref()
			.map_or(self.list.head, |node| node.next).as_mut()
			.map(|node| &mut node.value)
	}

	pub fn peek_prev(&mut self) -> Option<&mut T> {
		self.current.as_ref()
			.map_or(self.list.tail, |node| node.prev).as_mut()
			.map(|node| &mut node.value)
	}

	pub fn as_cursor(&self) -> Cursor<T> {
		Cursor { next_index: self.next_index, current: self.current, list: self.list }
	}

	pub fn insert_after(&mut self, item: T) {
		if let Some(node_before) = self.current.as_mut() {
			self.list.len += 1;
			let inserted = NodePtr::new(item, &self.current, &node_before.next);
			if let Some(node_after) = node_before.next.as_mut() {
				node_after.prev = inserted;
			} else {
				self.list.tail = inserted;
			}
			node_before.next = inserted;
		} else {
			self.list.push_front(item);
		}
	}

	pub fn insert_before(&mut self, item: T) {
		if let Some(node_after) = self.current.as_mut() {
			self.list.len += 1;
			let inserted = NodePtr::new(item, &node_after.prev, &self.current);
			if let Some(node_before) = node_after.prev.as_mut() {
				node_before.next = inserted;
			} else {
				self.list.head = inserted;
			}
			self.next_index += 1;
		} else {
			self.list.push_back(item);
		}
	}

	pub fn remove_current(&mut self) -> Option<T> {
		let mut boxed = self.current.into_box()?;
		self.list.len -= 1;
		match boxed.prev.as_mut() {
			Some(before) => before.next = boxed.next,
			None => self.list.head = boxed.next,
		}
		match boxed.next.as_mut() {
			Some(after) => after.prev = boxed.prev,
			None => self.list.tail = boxed.prev,
		}
		self.current = boxed.next;
		Some(boxed.value)
	}

	pub fn remove_current_as_list(&mut self) -> Option<LinkedList<T>> {
		let node = self.current.as_mut()?;
		let node_ptr = self.current;
		self.list.len -= 1;
		if let Some(before) = node.prev.as_mut() {
			before.next = node.next;
		}
		if let Some(after) = node.next.as_mut() {
			after.prev = node.prev;
		}
		self.current = node.next;
		Some(LinkedList { head: node_ptr, tail: node_ptr, len: 1 })
	}

	pub fn splice_after(&mut self, mut list: LinkedList<T>) {
		if list.is_empty() {
			return;
		} else if self.list.is_empty() {
			std::mem::swap(self.list, &mut list);
			return;
		} else if let Some(before) = self.current.as_mut() {
			if let Some(after) = before.next.as_mut() {
				after.prev = list.tail;
				list.tail.as_mut_unchecked().next = before.next;
			}
			before.next = list.head;
			list.head.as_mut_unchecked().prev = self.current;
		} else {
			self.list.head.as_mut_unchecked().prev = list.tail;
			list.tail.as_mut_unchecked().next = std::mem::replace(&mut self.list.head, list.head);
		}
		self.list.len += list.len();
		std::mem::forget(list);
	}

	pub fn splice_before(&mut self, mut list: LinkedList<T>) {
		if list.is_empty() {
			return;
		} else if self.list.is_empty() {
			std::mem::swap(self.list, &mut list);
			return;
		} else if let Some(after) = self.current.as_mut() {
			if let Some(before) = after.prev.as_mut() {
				before.next = list.head;
				list.head.as_mut_unchecked().prev = after.prev;
			}
			after.prev = list.tail;
			list.tail.as_mut_unchecked().next = self.current;
			self.next_index += list.len();
		} else {
			self.list.tail.as_mut_unchecked().next = list.head;
			list.head.as_mut_unchecked().prev = std::mem::replace(&mut self.list.tail, list.tail);
		}
		self.list.len += list.len();
		std::mem::forget(list);
	}

	pub fn split_after(&mut self) -> LinkedList<T> {
		if let Some(node) = self.current.as_mut() {
			if let Some(next) = node.next.as_mut() {
				next.prev = Default::default();
				let head = std::mem::take(&mut node.next);
				let tail = std::mem::replace(&mut self.list.tail, self.current);
				let len = self.list.len() - self.next_index;
				self.list.len = self.next_index;
				LinkedList { head, tail, len }
			} else {
				Default::default()
			}
		} else {
			std::mem::take(&mut self.list)
		}
	}

	pub fn split_before(&mut self) -> LinkedList<T> {
		if let Some(node) = self.current.as_mut() {
			if let Some(prev) = node.prev.as_mut() {
				prev.next = Default::default();
				let tail = std::mem::take(&mut node.prev);
				let head = std::mem::replace(&mut self.list.head, self.current);
				let len = self.next_index - 1;
				self.list.len -= len;
				self.next_index = 1;
				LinkedList { head, tail, len }
			} else {
				Default::default()
			}
		} else {
			std::mem::take(&mut self.list)
		}
	}

	pub fn push_front(&mut self, elt: T) {
		self.list.push_front(elt);
		self.next_index += 1;
	}

	pub fn push_back(&mut self, elt: T) {
		self.list.push_back(elt);
	}

	pub fn pop_front(&mut self) -> Option<T> {
		if let Some(node) = self.current.as_ref() {
			if self.next_index == 1 {
				self.current = node.next;
			} else {
				self.next_index -= 1;
			}
			self.list._pop_front()
		} else {
			self.list.pop_front()
		}
	}

	pub fn pop_back(&mut self) -> Option<T> {
		if self.current.ptr.is_some() {
			if self.next_index == self.list.len() {
				self.current = Default::default();
			}
			self.list._pop_back()
		} else {
			self.list.pop_back()
		}
	}

	pub fn front(&self) -> Option<&T> {
		self.list.head.as_ref().map(|node| &node.value)
	}

	pub fn front_mut(&mut self) -> Option<&mut T> {
		self.list.head.as_mut().map(|node| &mut node.value)
	}

	pub fn back(&self) -> Option<&T> {
		self.list.tail.as_ref().map(|node| &node.value)
	}

	pub fn back_mut(&mut self) -> Option<&mut T> {
		self.list.tail.as_mut().map(|node| &mut node.value)
	}
}

impl<T: Debug> Debug for CursorMut<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("CursorMut")
			.field(&self.list)
			.field(&self.index())
			.finish()
	}
}

impl<T: Debug, F: FnMut(&mut T) -> bool> Debug for DrainFilter<'_, T, F> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("DrainFilter").field(&self.list).finish()
	}
}

impl<T, F: FnMut(&mut T) -> bool> Drop for DrainFilter<'_, T, F> {
	fn drop(&mut self) {
		self.for_each(drop);
	}
}

impl<T, F: FnMut(&mut T) -> bool> Iterator for DrainFilter<'_, T, F> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(node) = self.current.as_mut() {
			if !(self.pred)(&mut node.value) {
				self.current = node.next;
				continue;
			}
			self.current = node.next;
			let mut boxed = self.current.into_box()?;
			self.list.len -= 1;
			match boxed.prev.as_mut() {
				Some(before) => before.next = boxed.next,
				None => self.list.head = boxed.next,
			}
			match boxed.next.as_mut() {
				Some(after) => after.prev = boxed.prev,
				None => self.list.tail = boxed.prev,
			}
			return Some(boxed.value);
		}
		None
	}
}
