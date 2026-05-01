use alloc::vec::Vec;
use orx_split_vec::{Growth, SplitVec};

pub fn merge_arb_into_first_vec<T>(results: Vec<Vec<T>>) -> Vec<T> {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    let mut results = results.into_iter();
    match results.next() {
        None => Default::default(),
        Some(mut result) => {
            let additional = total_len - result.len();
            result.reserve(additional);
            for vec in results {
                result.extend(vec);
            }
            result
        }
    }
}

pub fn merge_arb_into_vec_new<T>(results: Vec<Vec<T>>, dst: &mut Vec<T>) {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    dst.reserve(total_len);
    for vec in results {
        dst.extend(vec);
    }
}

pub fn merge_arb_into_vec<T>(results: Vec<Vec<T>>, mut dst: Vec<T>) -> Vec<T> {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    dst.reserve(total_len);
    for vec in results {
        dst.extend(vec);
    }
    dst
}

pub fn merge_arb_into_split_vec_new<T, G: Growth>(results: Vec<Vec<T>>, dst: &mut SplitVec<T, G>) {
    for vec in results {
        dst.extend(vec);
    }
}

pub fn merge_arb_into_split_vec<T, G: Growth>(
    results: Vec<Vec<T>>,
    mut dst: SplitVec<T, G>,
) -> SplitVec<T, G> {
    for vec in results {
        dst.extend(vec);
    }
    dst
}
