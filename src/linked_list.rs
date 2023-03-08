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
		NodePtr { ptr: self.ptr }
	}
}

impl<T> Copy for NodePtr<T> {}

impl<T> Default for NodePtr<T> {
	fn default() -> Self {
		NodePtr { ptr: None }
	}
}

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
		self.len += std::mem::replace(&mut other.len, 0);
		let other_head = std::mem::replace(&mut other.head, Default::default());
		self.tail.as_mut_unchecked().next = other_head;
		other_head.as_mut_unchecked().prev = self.tail;
		self.tail = std::mem::replace(&mut other.tail, Default::default());
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

	pub fn pop_front(&mut self) -> Option<T> {
		let boxed = self.head.into_box()?;
		self.head = boxed.next;
		self.len -= 1;
		if let Some(new_head_node) = self.head.as_mut() {
			new_head_node.prev = Default::default();
		}
		Some(boxed.value)
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

	pub fn pop_back(&mut self) -> Option<T> {
		let boxed = self.tail.into_box()?;
		self.tail = boxed.prev;
		self.len -= 1;
		if let Some(new_tail_node) = self.tail.as_mut() {
			new_tail_node.next = Default::default();
		}
		Some(boxed.value)
	}

	pub fn split_off(&mut self, at: usize) -> LinkedList<T> {
		if at == 0 {
			return std::mem::take(self);
		} else if at == self.len() {
			return Self::new();
		}
		let right_len = self.len - at;
		self.len = at;
		// TODO optimize by going from back
		let mut node_ptr = self.head;
		for _ in 0..at {
			node_ptr = node_ptr.as_ref().unwrap().next;
		}
		let right_node = node_ptr.as_mut_unchecked();
		let right_tail = std::mem::replace(&mut self.tail,
			std::mem::replace(&mut right_node.prev, Default::default())
		);
		self.tail.as_mut_unchecked().next = Default::default();
		LinkedList { head: node_ptr, tail: right_tail, len: right_len }
	}
}

// TODO Cursor
// TODO other experimentals

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

unsafe impl<T: Send> Send for LinkedList<T> {}

unsafe impl<T: Sync> Sync for LinkedList<T> {}

// Iter

impl<T> Clone for Iter<'_, T> {
	fn clone(&self) -> Self {
		Self { ..*self }
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
}

unsafe impl<T: Sync> Send for Iter<'_, T> {}

unsafe impl<T: Sync> Sync for Iter<'_, T> {}

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
}

unsafe impl<T: Sync> Send for IterMut<'_, T> {}

unsafe impl<T: Sync> Sync for IterMut<'_, T> {}

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
}
