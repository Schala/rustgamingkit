use bitvec::prelude::*;

/// Pops the last N bits from a bit vector, returning an integer
pub fn vec_pop_n<O, T>(bv: &mut BitVec<O, T>, n: usize) -> usize
where
	O: BitOrder,
	T: BitStore,
{
	let mut out = 0;

	for i in 0..n {
		if let Some(v) = bv.pop() {
			if v == true {
				out |= 1 << i;
			}
		}
	}

	out
}

#[test]
fn test_pop_n() {
	use bitvec::bitvec;

	let mut bv = bitvec![0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0];
	assert_eq!(vec_pop_n(&mut bv, 12), 1516);
}
