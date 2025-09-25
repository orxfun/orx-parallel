allBenches=(
    chain_collect_map
    chain3_collect_map
    chain4_collect_map
    collect_filter
    collect_filtermap
    collect_flatmap
    collect_iter_into_par
    collect_long_chain
    collect_map_filter_hash_set
    collect_map_filter
    collect_map
    count_filtermap
    count_flatmap
    count_map_filter
    count_map
    drain_vec_collect_map_filter
    find_any
    find_flatmap
    find_iter_into_par
    find_map_filter
    find
    mut_for_each_iter
    mut_for_each_slice
    reduce_iter_into_par
    reduce_long_chain
    reduce_map_filter
    reduce_map
    reduce
    result_collect_map
    result_reduce_map
    sum_filtermap
    sum_flatmap
    sum_map_filter
    sum
    vec_deque_collect_map_filter
    vec_deque_collect_map_filter_owned
)

counter = 0

for t in ${allBenches[@]}; do
    counter=$((counter+1))
    echo -e "\n\n"
    echo ------------- $counter : $t ----------------------------------
    ./.scripts/run_benchmark.sh $t
done
