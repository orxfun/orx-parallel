use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

const N: usize = 1_000_000;
const ERROR_AT: usize = 567_890;

fn map(a: usize) -> Result<usize, String> {
    match a {
        ERROR_AT => Err(format!("no {a}")),
        _ => Ok(a),
    }
}

fn seq() {
    println!("\n\nSEQ");
    let bag = ConcurrentBag::new();
    let inputs = 0..N;
    let result: Result<Vec<_>, _> = inputs
        .map(|x| {
            bag.push(x);
            map(x)
        })
        .collect();
    println!("result = {result:?}");
    let touched = bag.into_inner().to_vec();
    let num_touched = touched.len();
    println!("num_touched = {num_touched:?}");
}

fn rayon() {
    use rayon::iter::*;

    println!("\n\nRAYON");
    let bag = ConcurrentBag::new();
    let inputs = 0..N;
    let result: Result<Vec<_>, _> = inputs
        .into_par_iter()
        .map(|x| {
            bag.push(x);
            map(x)
        })
        .collect();
    println!("result = {result:?}");
    let touched = bag.into_inner().to_vec();
    let num_touched = touched.len();
    println!("num_touched = {num_touched:?}");
}

// fn orx() {
//     use orx_parallel::*;
//     println!("\n\nORX");
//     let bag = ConcurrentBag::new();
//     let o_bag = ConcurrentOrderedBag::new();
//     let inputs = 0..N;
//     let result: Result<Vec<_>, _> = inputs
//         .par()
//         .find(|x| {
//             let mapped = map(*x);
//             match mapped {
//                 Ok(x) => o_bag.push(x),
//                 Err(e) => {}
//             }
//         })
//         .collect();
//     println!("result = {result:?}");
//     let touched = bag.into_inner().to_vec();
//     let num_touched = touched.len();
//     println!("num_touched = {num_touched:?}");
// }

fn main() {
    seq();
    rayon();
}
