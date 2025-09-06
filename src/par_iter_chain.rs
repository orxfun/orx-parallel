use crate::{DefaultRunner, IntoParIter, ParIter, ParallelRunner};
use orx_concurrent_iter::{
    ConcurrentIter, ExactSizeConcurrentIter, IntoConcurrentIter,
    chain::{ChainKnownLenI, ChainUnknownLenI},
};

// /// Parallel iterator that can be chained with other parallel iterators.
// pub trait ChainableParIter<R = DefaultRunner>
// where
//     R: ParallelRunner,
//     Self: ParIter<R>,
//     Self: IntoConcurrentIter<Item = <Self as ParIter<R>>::Item>,
// {
//     /// Type of the concurrent iterator which provides input data.
//     type InputConIter: ConcurrentIter;

//     /// Creates a chain of this and `other` parallel iterators.
//     ///
//     /// It is preferable to call [`chain`] over `chain_inexact` whenever the input concurrent iterator
//     /// of this parallel iterator implements `ExactSizeConcurrentIter`.
//     ///
//     /// [`chain`]: ChainableParIter::chain
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use orx_parallel::*;
//     ///
//     /// let s1 = "abcxyz".chars().filter(|x| !['x', 'y', 'z'].contains(x)); // inexact iter
//     /// let s2 = vec!['d', 'e', 'f'];
//     ///
//     /// let chars: Vec<_> = s1.iter_into_par().chain_inexact(s2).collect();
//     /// assert_eq!(chars, vec!['a', 'b', 'c', 'd', 'e', 'f']);
//     /// ```
//     fn chain_inexact<C>(
//         self,
//         other: C,
//     ) -> impl ChainableParIter<
//         R,
//         InputConIter = ChainUnknownLenI<Self::InputConIter, C::IntoIter>,
//         Item = Self::Item,
//     >
//     where
//         C: IntoParIter<Item = <Self::InputConIter as ConcurrentIter>::Item>;

//     /// Creates a chain of this and `other` parallel iterators.
//     ///
//     /// It is preferable to call `chain` over [`chain_inexact`] whenever the input concurrent iterator
//     /// of this parallel iterator implements `ExactSizeConcurrentIter`.
//     ///
//     /// [`chain_inexact`]: ChainableParIter::chain_inexact
//     ///
//     /// # Examples
//     ///
//     /// ```
//     /// use orx_parallel::*;
//     ///
//     /// let s1 = "abc".chars(); // exact iter
//     /// let s2 = vec!['d', 'e', 'f'];
//     ///
//     /// let chain = s1.iter_into_con_iter().chain_inexact(s2);
//     ///
//     /// assert_eq!(chain.next(), Some('a'));
//     /// assert_eq!(chain.next(), Some('b'));
//     /// assert_eq!(chain.next(), Some('c'));
//     /// assert_eq!(chain.next(), Some('d'));
//     /// assert_eq!(chain.next(), Some('e'));
//     /// assert_eq!(chain.next(), Some('f'));
//     /// assert_eq!(chain.next(), None);
//     /// ```
//     fn chain<C>(
//         self,
//         other: C,
//     ) -> impl ChainableParIter<
//         R,
//         InputConIter = ChainKnownLenI<Self::InputConIter, C>,
//         Item = <Self as ParIter<R>>::Item,
//     >
//     where
//         Self::InputConIter: ExactSizeConcurrentIter,
//         C: ConcurrentIter<Item = <Self::InputConIter as ConcurrentIter>::Item>;
// }

// #[test]
// fn abc() {
//     use crate::*;

//     // let chars: Vec<_> = vec!['a', 'b', 'c']
//     //     .into_par() // exact iter
//     //     .chain(vec!['d', 'e', 'f'])
//     //     .collect();
//     // assert_eq!(chars, vec!['a', 'b', 'c', 'd', 'e', 'f']);

//     // let chars: Vec<_> = ['a', 'b', 'c']
//     //     .into_iter()
//     //     .iter_into_par() // exact iter
//     //     .chain(vec!['d', 'e', 'f'])
//     //     .collect();
//     // assert_eq!(chars, vec!['a', 'b', 'c', 'd', 'e', 'f']);

//     let chars: Vec<_> = vec!['a', 'b', 'c']
//         .into_par() // exact iter
//         .chain("def".chars().iter_into_par())
//         .collect();
//     assert_eq!(chars, vec!['a', 'b', 'c', 'd', 'e', 'f']);
// }
