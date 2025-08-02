use orx_parallel::*;
use std::sync::mpsc::channel;

fn main() {
    let (sender, receiver) = channel();

    (0..5)
        .into_par()
        .using(sender)
        .for_each_u(|s, x| s.send(x).unwrap());

    let mut res: Vec<_> = receiver.iter().collect();

    res.sort();

    assert_eq!(&res[..], &[0, 1, 2, 3, 4])
}
