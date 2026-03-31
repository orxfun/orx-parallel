// use orx_parallel::*;
// use std::sync::mpsc::channel;

// // taken from rayon's documentation:
// // https://docs.rs/rayon/1.10.0/rayon/iter/trait.ParallelIterator.html#method.map
// fn for_each() {
//     let (sender, receiver) = channel();

//     (0..5)
//         .into_par()
//         .using_clone(sender)
//         .for_each(|s, x| s.send(x).unwrap());

//     let mut res: Vec<_> = receiver.iter().collect();

//     res.sort();

//     assert_eq!(&res[..], &[0, 1, 2, 3, 4])
// }

// // taken from rayon's documentation:
// // https://docs.rs/rayon/1.10.0/rayon/iter/trait.ParallelIterator.html#method.map
// fn map() {
//     let (sender, receiver) = channel();

//     let a: Vec<_> = (0..5)
//         .into_par() // iterating over i32
//         .using_clone(sender)
//         .map(|s, x| {
//             s.send(x).unwrap(); // sending i32 values through the channel
//             x // returning i32
//         })
//         .collect(); // collecting the returned values into a vector

//     let mut b: Vec<_> = receiver
//         .iter() // iterating over the values in the channel
//         .collect(); // and collecting them
//     b.sort();

//     assert_eq!(a, b);
// }

// fn main() {
//     for_each();
//     map();
// }

fn main() {}
