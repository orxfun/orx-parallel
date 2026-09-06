use orx_parallel::*;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

fn fetch_user_activity() -> Vec<&'static str> {
    thread::sleep(Duration::from_millis(300));
    vec!["login", "view_item", "checkout"]
}

fn compute_financial_summary() -> (f64, usize) {
    thread::sleep(Duration::from_millis(500));
    (149.99, 3)
}

fn generate_recommendations() -> Vec<&'static str> {
    thread::sleep(Duration::from_millis(200));
    vec!["wireless_mouse", "mechanical_keyboard"]
}

fn main() {
    println!("=== Ad-Hoc Parallel Tasks Example ===\n");

    let activity_logs = Mutex::new(Vec::new());
    let financial_summary = Mutex::new((0.0, 0));
    let recommendations = Mutex::new(Vec::new());

    let start_time = Instant::now();

    // Bundle ad-hoc independent tasks using the `tasks!` macro
    let tasks = tasks![
        || {
            let start = Instant::now();
            let res = fetch_user_activity();
            println!(
                "Task 1 (User Activity - 300ms) finished in {:?}",
                start.elapsed()
            );
            *activity_logs.lock().unwrap() = res;
        },
        || {
            let start = Instant::now();
            let res = compute_financial_summary();
            println!(
                "Task 2 (Financial Summary - 500ms) finished in {:?}",
                start.elapsed()
            );
            *financial_summary.lock().unwrap() = res;
        },
        || {
            let start = Instant::now();
            let res = generate_recommendations();
            println!(
                "Task 3 (Recommendations - 200ms) finished in {:?}",
                start.elapsed()
            );
            *recommendations.lock().unwrap() = res;
        },
    ];

    // Execute all tasks in parallel on the thread pool
    Pool::global().run_all(tasks);

    let total_duration = start_time.elapsed();

    println!("\n--- Collected Results ---");
    println!("Activity logs: {:?}", activity_logs.into_inner().unwrap());
    println!(
        "Financial summary: {:?}",
        financial_summary.into_inner().unwrap()
    );
    println!(
        "Recommendations: {:?}",
        recommendations.into_inner().unwrap()
    );

    println!("\nTotal parallel execution time: {:?}", total_duration);
}
