original_bench=find
bench=$1

sed -i "s/$original_bench/$bench/g" Cargo.toml

rm -f benches/results/$bench.txt

cargo bench >> benches/results/$bench.txt

sed -i "s/$bench/$original_bench/g" Cargo.toml
