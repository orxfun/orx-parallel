use orx_parallel::*;

fn map(a: usize) -> Result<usize, String> {
    match a {
        6 | 8 => Err(format!("no {a}")),
        _ => Ok(a),
    }
}

fn main() {
    let inputs = 0..10;
    let results: Result<Vec<_>, _> = inputs.map(map).collect();
    println!("{results:?}");

    let inputs = 0..10;
    let par = inputs.par();

    let results: Result<Vec<_>, _> = inputs.map(map).collect();
    println!("{results:?}")
}
