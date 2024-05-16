pub(crate) fn chunk_size(
    requested_chunk_size: Option<usize>,
    len: Option<usize>,
    num_threads: usize,
) -> usize {
    requested_chunk_size.unwrap_or_else(|| match len {
        Some(len) => {
            let average_elements_per_thread = len / num_threads;
            match average_elements_per_thread {
                x if x > 64 => 64,
                x if x > 32 => 32,
                x if x > 16 => 16,
                x if x > 4 => 4,
                _ => 1,
            }
        }
        None => 1,
    })
}

pub(crate) fn num_threads(requested_num_threads: Option<usize>) -> usize {
    requested_num_threads.unwrap_or_else(|| {
        let max_threads = std::thread::available_parallelism();
        match max_threads {
            Err(e) => {
                debug_assert!(false, "max-threads failed: {}", e);
                4
            }
            Ok(x) => x.into(),
        }
    })
}
