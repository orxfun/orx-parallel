use orx_parallel::*;

// Each pending order is represented as (order_id, amount_cents).
type Order = (u32, u64);

fn pending_orders() -> Vec<Order> {
    (1..=1000).map(|id| (id, 1_000 + id as u64 * 10)).collect()
}

fn main() {
    let mut queue: Vec<Order> = pending_orders();
    let total_in_queue: u64 = queue.iter().map(|(_, amt)| amt).sum();

    // Dispatch the first 200 orders: drain them from the queue and compute
    // their combined value in parallel. The remaining orders stay in the queue.
    let batch_size = 200;
    let batch_total: u64 = queue
        .par_drain(..batch_size)
        .map(|(_, amount)| amount)
        .reduce(|a, b| a + b)
        .unwrap_or(0);

    // The dispatched orders are gone from the queue.
    assert_eq!(queue.len(), 1000 - batch_size);

    // The remaining queue total equals the original minus what was dispatched.
    let remaining_total: u64 = queue.iter().map(|(_, amt)| amt).sum();
    assert_eq!(batch_total + remaining_total, total_in_queue);
}
