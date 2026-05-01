mod merge_arbitrary_into;
mod merge_ordered_into;

pub use merge_arbitrary_into::{
    merge_arb_into_first_vec, merge_arb_into_split_vec, merge_arb_into_vec,
};
pub use merge_ordered_into::{
    merge_ord_into_split_vec, merge_ord_into_vec, merge_ord_into_vec_new,
};
