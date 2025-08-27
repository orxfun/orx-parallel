# allBenches=(collect_filter sum)

allBenches=(
    collect_filter
    collect_filtermap
    collect_flatmap
    collect_iter_into_par
    collect_long_chain
    collect_map
    collect_map_filter
    collect_map_filter_hash_set
    collect_result
    count_filtermap
    count_flatmap
    count_map
    count_map_filter
    drain_vec_collect_map_filter
    find
    find_any
    find_flatmap
    find_iter_into_par
    find_map_filter
    mut_for_each_iter
    mut_for_each_slice
    reduce
    reduce_iter_into_par
    reduce_long_chain
    reduce_map
    reduce_map_filter
    sum
    sum_filtermap
    sum_flatmap
    sum_map_filter
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
