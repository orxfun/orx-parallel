# allBenches=(collect_filter sum)

allBenches=(collect_filter collect_map_filter count_map_filter find_map_filter reduce_map sum_map_filter collect_filtermap collect_map_filter_hash_set drain_vec_collect_map_filter mut_for_each_iter reduce_map_filter vec_deque_collect_map_filter collect_flatmap collect_result find mut_for_each_slice vec_deque_collect_map_filter_owned collect_iter_into_par count_filtermap find_any reduce sum collect_long_chain count_flatmap find_flatmap reduce_iter_into_par sum_filtermap collect_map count_map find_iter_into_par reduce_long_chain sum_flatmap)

counter = 0

for t in ${allBenches[@]}; do
    counter=$((counter+1))
    echo -e "\n\n"
    echo ------------- $counter : $t ----------------------------------
    ./.scripts/run_benchmark.sh $t
done
