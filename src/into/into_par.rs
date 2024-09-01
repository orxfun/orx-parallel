use crate::par::par_empty::ParEmpty;
use iter::atomic_iter::AtomicIter;
use orx_concurrent_iter::*;
use std::ops::{Add, Range, Sub};

/// Conversion into a parallel iterator.
///
/// Every type implementing [`orx_concurrent_iter::ConcurrentIter`] or [`orx_concurrent_iter::IntoConcurrentIter`] also implements `IntoPar`.
/// These types include common collections/views such as range, vector or slice.
///
/// See [`crate::IterIntoPar`] for conversion of any regular iterator into parallel iterator.
///
/// Converting into a parallel iterator is achieved using the `into_par()` method.
///
/// # Examples
///
/// ```rust
/// use orx_parallel::*;
///
/// let seq: usize = (0..1024).sum();
/// let par = (0..1024).into_par().sum();
/// assert_eq!(par, seq);
///
/// let seq = vec![10; 42].into_iter().sum();
/// let par = vec![10; 42].into_par().sum();
/// assert_eq!(par, seq);
///
/// let seq = (10..420).filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
/// let par = (10..420).into_par().filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
/// assert_eq!(par, seq);
///
/// let names = ["john", "doe", "foo", "bar"].map(String::from);
/// let seq = names.iter().map(|x| x.len()).reduce(|a, b| a + b);
/// let par = names.as_slice().into_par().map(|x| x.len()).reduce(|a, b| a + b);
/// assert_eq!(par, seq);
/// ```
pub trait IntoPar {
    /// Underlying concurrent iterator which provides the input elements to the defined parallel computation.
    type ConIter: ConcurrentIter;

    /// Conversion into a parallel iterator.
    ///
    /// Every type implementing [`orx_concurrent_iter::ConcurrentIter`] or [`orx_concurrent_iter::IntoConcurrentIter`] also implements `IntoPar`.
    /// These types include common collections/views such as range, vector or slice.
    ///
    /// See [`crate::IterIntoPar`] for conversion of any regular iterator into parallel iterator.
    ///
    /// Converting into a parallel iterator is achieved using the `into_par()` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let seq = vec![10; 42].into_iter().sum();
    /// let par = vec![10; 42].into_par().sum();
    /// assert_eq!(par, seq);
    ///
    /// let seq = (10..420).filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
    /// let par = (10..420).into_par().filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
    /// assert_eq!(par, seq);
    ///
    /// let names = ["john", "doe", "foo", "bar"].map(String::from);
    /// let seq = names.iter().map(|x| x.len()).reduce(|a, b| a + b);
    /// let par = names.as_slice().into_par().map(|x| x.len()).reduce(|a, b| a + b);
    /// assert_eq!(par, seq);
    /// ```
    fn into_par(self) -> ParEmpty<Self::ConIter>;
}

// con-iter
impl<T: Send + Sync, Iter> IntoPar for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type ConIter = ConIterOfIter<T, Iter>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}

// range
impl<Idx> IntoPar for Range<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
    Range<Idx>: Iterator<Item = Idx>,
{
    type ConIter = ConIterOfRange<Idx>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.con_iter())
    }
}
impl<Idx> IntoPar for ConIterOfRange<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
    Range<Idx>: Iterator<Item = Idx>,
{
    type ConIter = ConIterOfRange<Idx>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}

// slice
impl<'a, T: Send + Sync> IntoPar for &'a [T] {
    type ConIter = ConIterOfSlice<'a, T>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.into_con_iter())
    }
}

impl<'a, T: Send + Sync> IntoPar for ConIterOfSlice<'a, T> {
    type ConIter = ConIterOfSlice<'a, T>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}

// cloned

impl<'a, T: Send + Sync + Clone, C: AtomicIter<&'a T> + ConcurrentIter<Item = &'a T>> IntoPar
    for Cloned<'a, T, C>
{
    type ConIter = Cloned<'a, T, C>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}

// vec
impl<T: Send + Sync> IntoPar for Vec<T> {
    type ConIter = ConIterOfVec<T>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.into_con_iter())
    }
}
impl<T: Send + Sync> IntoPar for ConIterOfVec<T> {
    type ConIter = ConIterOfVec<T>;
    fn into_par(self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self)
    }
}

// std collections

mod impl_std_collections {

    use crate::{par::par_empty::ParEmpty, IntoPar};
    use orx_concurrent_iter::*;
    use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    };

    impl<T: Send + Sync> IntoPar for VecDeque<T> {
        type ConIter = ConIterOfIter<T, std::collections::vec_deque::IntoIter<T>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<T: Send + Sync> IntoPar for BTreeSet<T> {
        type ConIter = ConIterOfIter<T, std::collections::btree_set::IntoIter<T>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<T: Send + Sync> IntoPar for HashSet<T> {
        type ConIter = ConIterOfIter<T, std::collections::hash_set::IntoIter<T>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<K: Send + Sync, V: Send + Sync> IntoPar for BTreeMap<K, V> {
        type ConIter = ConIterOfIter<(K, V), std::collections::btree_map::IntoIter<K, V>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<K: Send + Sync, V: Send + Sync> IntoPar for HashMap<K, V> {
        type ConIter = ConIterOfIter<(K, V), std::collections::hash_map::IntoIter<K, V>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<T: Send + Sync> IntoPar for LinkedList<T> {
        type ConIter = ConIterOfIter<T, std::collections::linked_list::IntoIter<T>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }

    impl<T: Send + Sync> IntoPar for BinaryHeap<T> {
        type ConIter = ConIterOfIter<T, std::collections::binary_heap::IntoIter<T>>;
        fn into_par(self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.into_iter().into_con_iter())
        }
    }
}
