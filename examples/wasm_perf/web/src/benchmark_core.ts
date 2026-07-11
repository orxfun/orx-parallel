export type VariantName = "rayon" | "orx-rayon" | "orx";

export type VariantRunner = {
    init_parallel_runtime: (numThreads: number) => Promise<unknown>;
    run_best_tour_par: (
        iterations: number,
        seed: bigint,
        threads: number,
        numCities: number,
        startIndex: bigint,
    ) => SearchChunkResult;
};

export type SearchChunkResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

export type BenchmarkConfig = {
    threads: number;
    cityCounts: number[];
    iterationCounts: number[];
    warmups: number;
    runs: number;
    seed: bigint;
};

export type BenchmarkRow = {
    variant: VariantName;
    threads: number;
    cities: number;
    iterations: number;
    medianMs: number;
    meanMs: number;
    minMs: number;
    maxMs: number;
    medianIps: number;
    meanIps: number;
    samplesMs: number[];
};

export type BenchmarkReport = {
    config: {
        threads: number;
        cityCounts: number[];
        iterationCounts: number[];
        warmups: number;
        runs: number;
        seed: string;
    };
    rows: BenchmarkRow[];
};

export function parseCsvInts(input: string, fallback: number[]): number[] {
    const values = input
        .split(",")
        .map((x) => Number.parseInt(x.trim(), 10))
        .filter((x) => Number.isFinite(x));

    if (values.length === 0) {
        return fallback;
    }

    return Array.from(new Set(values.map((x) => Math.max(1, x))));
}

export function summarize(samplesMs: number[], iterations: number): Omit<BenchmarkRow, "variant" | "threads" | "cities" | "iterations"> {
    const sorted = [...samplesMs].sort((a, b) => a - b);
    const medianMs = percentile(sorted, 50);
    const sumMs = samplesMs.reduce((acc, x) => acc + x, 0);
    const meanMs = samplesMs.length > 0 ? sumMs / samplesMs.length : 0;
    const minMs = sorted[0] ?? 0;
    const maxMs = sorted[sorted.length - 1] ?? 0;
    const medianIps = ips(iterations, medianMs);
    const meanIps = ips(iterations, meanMs);

    return {
        medianMs,
        meanMs,
        minMs,
        maxMs,
        medianIps,
        meanIps,
        samplesMs,
    };
}

export async function runVariantMatrix(
    variant: VariantName,
    runner: VariantRunner,
    cfg: BenchmarkConfig,
    onProgress: (message: string) => void,
): Promise<BenchmarkRow[]> {
    const rows: BenchmarkRow[] = [];

    for (const cities of cfg.cityCounts) {
        for (const iterations of cfg.iterationCounts) {
            onProgress(`Running ${variant} for cities=${cities}, iterations=${iterations}...`);

            let startIndex = 0n;
            for (let i = 0; i < cfg.warmups; i++) {
                runner.run_best_tour_par(iterations, cfg.seed, cfg.threads, cities, startIndex);
                startIndex += BigInt(iterations);
            }

            const samplesMs: number[] = [];
            for (let i = 0; i < cfg.runs; i++) {
                const result = runner.run_best_tour_par(iterations, cfg.seed, cfg.threads, cities, startIndex);
                samplesMs.push(result.elapsed_ms);
                startIndex += BigInt(iterations);
            }

            const stats = summarize(samplesMs, iterations);
            rows.push({
                variant,
                threads: cfg.threads,
                cities,
                iterations,
                ...stats,
            });
        }
    }

    return rows;
}

export function formatReportText(report: BenchmarkReport): string {
    const lines: string[] = [];
    lines.push("WASM benchmark report");
    lines.push("");
    lines.push(`threads: ${report.config.threads}`);
    lines.push(`cityCounts: ${report.config.cityCounts.join(", ")}`);
    lines.push(`iterationCounts: ${report.config.iterationCounts.join(", ")}`);
    lines.push(`warmups: ${report.config.warmups}`);
    lines.push(`runs: ${report.config.runs}`);
    lines.push(`seed: ${report.config.seed}`);
    lines.push("");
    lines.push("variant | cities | iterations | median_ms | mean_ms | median_ips | mean_ips");
    lines.push("--- | ---: | ---: | ---: | ---: | ---: | ---:");

    for (const row of report.rows) {
        lines.push(
            `${row.variant} | ${row.cities} | ${row.iterations} | ${row.medianMs.toFixed(2)} | ${row.meanMs.toFixed(2)} | ${row.medianIps.toFixed(0)} | ${row.meanIps.toFixed(0)}`,
        );
    }

    lines.push("");
    lines.push("Raw JSON:");
    lines.push(JSON.stringify(report, null, 2));
    return lines.join("\n");
}

function percentile(sorted: number[], p: number): number {
    if (sorted.length === 0) {
        return 0;
    }
    const idx = Math.floor(((sorted.length - 1) * p) / 100);
    return sorted[idx];
}

function ips(iterations: number, elapsedMs: number): number {
    return iterations / Math.max(elapsedMs / 1000, 1e-9);
}
