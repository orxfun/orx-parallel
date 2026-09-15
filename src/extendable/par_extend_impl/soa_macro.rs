/// Implements `ParExtendCore` for an SoA type with tuple item arity.
#[macro_export]
macro_rules! impl_par_extend_for_soa {
	($soa:ident, 1) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1], [a1], [b1]);
	};
	($soa:ident, 2) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2], [a1, a2], [b1, b2]);
	};
	($soa:ident, 3) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3], [a1, a2, a3], [b1, b2, b3]);
	};
	($soa:ident, 4) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4], [a1, a2, a3, a4], [b1, b2, b3, b4]);
	};
	($soa:ident, 5) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5], [a1, a2, a3, a4, a5], [b1, b2, b3, b4, b5]);
	};
	($soa:ident, 6) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6], [a1, a2, a3, a4, a5, a6], [b1, b2, b3, b4, b5, b6]);
	};
	($soa:ident, 7) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7], [a1, a2, a3, a4, a5, a6, a7], [b1, b2, b3, b4, b5, b6, b7]);
	};
	($soa:ident, 8) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7, T8], [a1, a2, a3, a4, a5, a6, a7, a8], [b1, b2, b3, b4, b5, b6, b7, b8]);
	};
	($soa:ident, 9) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7, T8, T9], [a1, a2, a3, a4, a5, a6, a7, a8, a9], [b1, b2, b3, b4, b5, b6, b7, b8, b9]);
	};
	($soa:ident, 10) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], [a1, a2, a3, a4, a5, a6, a7, a8, a9, a10], [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10]);
	};
	($soa:ident, 11) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], [a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11], [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11]);
	};
	($soa:ident, 12) => {
		$crate::impl_par_extend_for_soa!(@impl $soa, [T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], [a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12], [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12]);
	};
	(@impl $soa:ident, [$($t:ident),+], [$($a:ident),+], [$($b:ident),+]) => {
		impl<$($t: Send),+> $crate::collectables::ParExtendCore<($($t),+)> for $soa<$($t),+> {
			type ThreadValues = Self;

			type OrderedThreadValues = $crate::collectables::par_extend_impl::utils::ColAndPos<Self>;

			fn new_thread_values() -> Self::ThreadValues {
				Self::new()
			}

			fn new_ordered_thread_values() -> Self::OrderedThreadValues {
				Default::default()
			}

			fn add_thread_value(collected: &mut Self::ThreadValues, value: ($($t),+)) {
				collected.push(value);
			}

			fn add_thread_values(
				collected: &mut Self::ThreadValues,
				values: impl IntoIterator<Item = ($($t),+)>,
			) {
				collected.extend(values)
			}

			fn add_ordered_thread_value(
				collected: &mut Self::OrderedThreadValues,
				idx: usize,
				value: ($($t),+),
			) {
				collected.values.push(value);
				collected.positions.push($crate::collectables::par_extend_impl::utils::IdxLen { idx, len: 1 });
			}

			fn add_ordered_thread_values(
				collected: &mut Self::OrderedThreadValues,
				idx: usize,
				values: impl IntoIterator<Item = ($($t),+)>,
			) {
				let len_begin = collected.values.len();
				collected.values.extend(values);

				let len = collected.values.len() - len_begin;
				if len > 0 {
					collected.positions.push($crate::collectables::par_extend_impl::utils::IdxLen { idx, len });
				}
			}

			fn add_ordered_thread_optionals(
				collected: &mut Self::OrderedThreadValues,
				idx: usize,
				values: impl IntoIterator<Item = Option<($($t),+)>>,
			) -> Option<()> {
				let len_begin = collected.values.len();
				for value in values {
					collected.values.push(value?);
				}

				let len = collected.values.len() - len_begin;
				if len > 0 {
					collected.positions.push($crate::collectables::par_extend_impl::utils::IdxLen { idx, len });
				}

				Some(())
			}

			fn add_ordered_thread_fallibles<E>(
				collected: &mut Self::OrderedThreadValues,
				idx: usize,
				values: impl IntoIterator<Item = Result<($($t),+), E>>,
			) -> Result<(), E> {
				let len_begin = collected.values.len();
				for value in values {
					collected.values.push(value?);
				}

				let len = collected.values.len() - len_begin;
				if len > 0 {
					collected.positions.push($crate::collectables::par_extend_impl::utils::IdxLen { idx, len });
				}

				Ok(())
			}

			#[inline(always)]
			fn add_one(&mut self, value: ($($t),+)) {
				self.push(value);
			}

			fn extend_merge_infallibles(&mut self, results: ::alloc::vec::Vec<Self::ThreadValues>) {
				let collected_len: usize = results.iter().map(|x| x.len()).sum();
				self.reserve(collected_len);
				for result in results {
					self.extend(result);
				}
			}

			fn extend_merge_ordered_infallibles(&mut self, mut results: ::alloc::vec::Vec<Self::OrderedThreadValues>) {
				use ::orx_priority_queue::PriorityQueue;

				let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
				self.reserve(collected_len);
				let initial_len = self.len();
				let total_len = initial_len + collected_len;

				let mut queue = ::orx_priority_queue::BinaryHeap::with_capacity(results.len());
				let mut pos_indices = ::alloc::vec![0; results.len()];

				for (t, vec) in results.iter().enumerate() {
					if let Some(pos) = vec.positions.first() {
						let node = $crate::collectables::par_extend_impl::vec::ThBegLen::new(t, 0, pos.len);
						queue.push(node, pos.idx);
					}
				}
				let mut curr_t = queue.pop_node();
				let mut ptr_dst = unsafe { mut_ptr_add(self.as_mut_ptr(), initial_len) };

				while let Some($crate::collectables::par_extend_impl::vec::ThBegLen { th, beg, len }) = curr_t {
					let ptr_src = unsafe { ptr_add(results[th].values.as_ptr(), beg) };
					unsafe { copy_nonoverlapping(ptr_src, ptr_dst, len) };

					pos_indices[th] += 1;
					curr_t = match results[th].positions.get(pos_indices[th]) {
						Some(pos) => {
							let beg = beg + len;
							let node = $crate::collectables::par_extend_impl::vec::ThBegLen::new(th, beg, pos.len);
							Some(queue.push_then_pop(node, pos.idx).0)
						}
						None => queue.pop_node(),
					};

					ptr_dst = unsafe { mut_ptr_add(ptr_dst, len) };
				}

				for vec in results.iter_mut() {
					unsafe { vec.values.set_len(0) };
				}

				unsafe { self.set_len(total_len) };
			}
		}

		unsafe fn ptr_add<$($t),+>(($($a),+): ($( *const $t ),+), count: usize) -> ($( *const $t ),+) {
			($(unsafe { $a.add(count) }),+)
		}

		unsafe fn mut_ptr_add<$($t),+>(($($a),+): ($( *mut $t ),+), count: usize) -> ($( *mut $t ),+) {
			($(unsafe { $a.add(count) }),+)
		}

		unsafe fn copy_nonoverlapping<$($t),+>(
			ptr_src: ($( *const $t ),+),
			ptr_dst: ($( *mut $t ),+),
			len: usize,
		) {
			let ($($a),+) = ptr_src;
			let ($($b),+) = ptr_dst;
			$(unsafe { $b.copy_from_nonoverlapping($a, len) };)+
		}
	};
}
