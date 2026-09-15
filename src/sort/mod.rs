#[cfg(test)]
mod tests;

#[cfg(feature = "experimental")]
mod sort_slice;
#[cfg(feature = "experimental")]
pub use sort_slice::par_experimental_sort;
