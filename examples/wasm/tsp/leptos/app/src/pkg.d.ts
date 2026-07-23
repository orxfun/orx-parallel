declare module "../pkg/components.js" {
    export default function init(): Promise<void>;

    export function start_app(): void;
    export function init_parallel_runtime(num_threads: number): Promise<void>;
    export function locations(seed: bigint, num_cities: number): unknown;
    export function run_search(
        parallelize: boolean,
        iterations: number,
        seed: bigint,
        threads: number,
        chunk_size: number,
        locations: unknown
    ): unknown;
}
