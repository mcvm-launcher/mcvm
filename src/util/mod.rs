/// Utilities for working with hashes and checksums
pub mod hash;

use rand::Rng;

/// Selects a random set of n elements from a list. The return slice will not necessarily be of n length
pub fn select_random_n_items_from_list<T>(list: &[T], n: usize) -> Vec<&T> {
	let mut indices: Vec<usize> = (0..list.len()).collect();
	let mut rng = rand::thread_rng();
	let mut chosen = Vec::new();
	for _ in 0..n {
		if indices.is_empty() {
			break;
		}

		let index = rng.gen_range(0..indices.len());
		let index = indices.remove(index);
		chosen.push(&list[index]);
	}

	chosen
}
