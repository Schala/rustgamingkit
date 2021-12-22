pub struct SuffixArray {
	sa: Vec<i32>,
	phi: Vec<i32>,
}

impl SuffixArray {
	#[inline]
	pub fn lpf(&mut self) -> &mut SuffixArray {
		if self.phi.is_empty() {
			self.phi = phi(sa.as_slice());
		}

		&mut self
	}
}

/// Longest previous factor
#[inline]
fn lpf(lengths: &mut [i32], offsets: &mut [i32], phi: &[i32], plcp: &[i32], input: &[u8]) {

}

/// Generates an auxiliary data structure for PLCP and LPF computation
#[inline]
fn phi(sa: &[i32]) -> Vec<i32> {
	let mut res = Vec::from(sa);

	res[sa[0]] = 0;
	for i in 1..sa.len() {
		res[sa[i]] = sa[i - 1];
	}

	res
}
