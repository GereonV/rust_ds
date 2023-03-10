use super::*;

#[test]
fn test_empty() {
	let mut m = LinkedList::<isize>::new();
	assert!(m.is_empty());
	assert_eq!(m.pop_front(), None);
	assert!(m.is_empty());
	assert_eq!(m.pop_back(), None);
	assert!(m.is_empty());
}
