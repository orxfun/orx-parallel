use crate::par::par_empty::ParEmpty;
use orx_concurrent_iter::*;

/// Non-consuming conversion into a parallel iterator.
///
/// Every type that implements `IntoConcurrentIter` implements `AsPar`.
///
/// See [`crate::IntoPar`] for consuming conversion of common collections into parallel iterator.
///
/// Converting into a parallel iterator is achieved using the `par()` method.
///
/// # Examples
///
/// ```rust
/// use orx_parallel::*;
///
/// let vec = vec![10usize; 42];
/// let seq = vec.iter().sum::<usize>();
/// let par = vec.par().copied().sum();
/// assert_eq!(par, seq);
///
/// let seq = (10..420).filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
/// let par = (10..420).par().filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
/// assert_eq!(par, seq);
///
/// let names = ["john", "doe", "foo", "bar"].map(String::from);
/// let seq = names.iter().map(|x| x.len()).reduce(|a, b| a + b);
/// let par = names.as_slice().into_par().map(|x| x.len()).reduce(|a, b| a + b);
/// assert_eq!(par, seq);
/// ```
pub trait AsPar<'a, T: Send + Sync> {
    /// Underlying concurrent iterator which provides the input elements to the defined parallel computation.
    type ConIter: ConcurrentIter;

    /// Non-consuming conversion into a parallel iterator.
    ///
    /// Every type that implements `IntoConcurrentIter` implements `AsPar`.
    ///
    /// See [`crate::IntoPar`] for consuming conversion of common collections into parallel iterator.
    ///
    /// Converting into a parallel iterator is achieved using the `par()` method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let vec = vec![10usize; 42];
    /// let seq = vec.iter().sum::<usize>();
    /// let par = vec.par().copied().sum();
    /// assert_eq!(par, seq);
    ///
    /// let seq = (10..420).filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
    /// let par = (10..420).par().filter(|x| x % 2 == 1).map(|x| 2 * x).sum();
    /// assert_eq!(par, seq);
    ///
    /// let names = ["john", "doe", "foo", "bar"].map(String::from);
    /// let seq = names.iter().map(|x| x.len()).reduce(|a, b| a + b);
    /// let par = names.as_slice().into_par().map(|x| x.len()).reduce(|a, b| a + b);
    /// assert_eq!(par, seq);
    /// ```
    fn par(&'a self) -> ParEmpty<Self::ConIter>;
}

// vec

impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for Vec<T> {
    type ConIter = ConIterOfSlice<'a, T>;

    fn par(&'a self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.con_iter())
    }
}

// array

impl<'a, const N: usize, T: Send + Sync + 'a> AsPar<'a, T> for [T; N] {
    type ConIter = ConIterOfSlice<'a, T>;

    fn par(&'a self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.con_iter())
    }
}

// slice

impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for &'a [T] {
    type ConIter = ConIterOfSlice<'a, T>;

    fn par(&'a self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.con_iter())
    }
}

// std collections

mod impl_std_collections {

    use crate::{par::par_empty::ParEmpty, AsPar};
    use orx_concurrent_iter::*;
    use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    };

    impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for VecDeque<T> {
        type ConIter = ConIterOfIter<&'a T, std::collections::vec_deque::Iter<'a, T>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for BTreeSet<T> {
        type ConIter = ConIterOfIter<&'a T, std::collections::btree_set::Iter<'a, T>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for HashSet<T> {
        type ConIter = ConIterOfIter<&'a T, std::collections::hash_set::Iter<'a, T>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, K: Send + Sync + 'a, V: Send + Sync + 'a> AsPar<'a, (K, V)> for BTreeMap<K, V> {
        type ConIter = ConIterOfIter<(&'a K, &'a V), std::collections::btree_map::Iter<'a, K, V>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, K: Send + Sync + 'a, V: Send + Sync + 'a> AsPar<'a, (K, V)> for HashMap<K, V> {
        type ConIter = ConIterOfIter<(&'a K, &'a V), std::collections::hash_map::Iter<'a, K, V>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for LinkedList<T> {
        type ConIter = ConIterOfIter<&'a T, std::collections::linked_list::Iter<'a, T>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }

    impl<'a, T: Send + Sync + 'a> AsPar<'a, T> for BinaryHeap<T> {
        type ConIter = ConIterOfIter<&'a T, std::collections::binary_heap::Iter<'a, T>>;

        fn par(&'a self) -> ParEmpty<Self::ConIter> {
            ParEmpty::new(self.iter().into_con_iter())
        }
    }
}
