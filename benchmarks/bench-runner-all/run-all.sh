cargo run --release -- \
    --path .. \
    --path-result ../../docs/bench-ui/results/ \
    --warmup-runs 20 \
    --actual-runs 100 \
    --threads 4 \
    --threads 8 \
    --threads 16 \
    --threads 20 \
    --threads 24 \
    --threads 28 \
    --threads 32 \
    # --categories algorithms \
    # --categories arbitrary_iter \
    --categories collect \
    --categories contention_merge \
    --categories early_exit \
    --categories fallible \
    --categories first \
    --categories heterogeneous \
    --categories memory_pressure \
    --categories reduce \
    --categories stateful_using \
    --categories throughput_linear
