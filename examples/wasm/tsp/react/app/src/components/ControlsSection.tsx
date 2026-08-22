import type { ChangeEvent } from "react";

type ControlsSectionProps = {
    iterations: number;
    threads: number;
    maxThreads: number;
    chunkSize: number;
    seed: number;
    numCities: number;
    isRunning: boolean;
    status: string;
    onIterationsChange: (value: number) => void;
    onThreadsChange: (value: number) => void;
    onChunkSizeChange: (value: number) => void;
    onSeedChange: (value: number) => void;
    onNumCitiesChange: (value: number) => void;
    onRun: () => void;
    onReset: () => void;
};

function readNumber(event: ChangeEvent<HTMLInputElement>, fallback: number): number {
    const parsed = event.currentTarget.valueAsNumber;
    return Number.isFinite(parsed) ? parsed : fallback;
}

export function ControlsSection(props: ControlsSectionProps) {
    return (
        <section className="card">
            <div className="control-panel">
                <div className="controls">
                    <label>
                        Number of cities
                        <input
                            id="numCities"
                            type="number"
                            min="5"
                            max="200"
                            value={props.numCities}
                            disabled={props.isRunning}
                            onChange={(event) => props.onNumCitiesChange(readNumber(event, props.numCities))}
                        />
                    </label>
                    <label>
                        Iterations
                        <input
                            id="iterations"
                            type="number"
                            min="1"
                            max="200000"
                            value={props.iterations}
                            disabled={props.isRunning}
                            onChange={(event) => props.onIterationsChange(readNumber(event, props.iterations))}
                        />
                    </label>
                    <label>
                        Threads (0..{props.maxThreads})
                        <input
                            id="threads"
                            type="number"
                            min="0"
                            max={props.maxThreads}
                            value={props.threads}
                            disabled={props.isRunning}
                            onChange={(event) => props.onThreadsChange(readNumber(event, props.threads))}
                        />
                    </label>
                    <label>
                        Chunk size
                        <input
                            id="chunkSize"
                            type="number"
                            min="0"
                            max="1048576"
                            value={props.chunkSize}
                            disabled={props.isRunning}
                            onChange={(event) => props.onChunkSizeChange(readNumber(event, props.chunkSize))}
                        />
                    </label>
                    <label>
                        Seed
                        <input
                            id="seed"
                            type="number"
                            min="1"
                            max="99999999"
                            value={props.seed}
                            disabled={props.isRunning}
                            onChange={(event) => props.onSeedChange(readNumber(event, props.seed))}
                        />
                    </label>
                </div>

                <div className="actions">
                    <button id="run" disabled={props.isRunning} onClick={props.onRun}>
                        Run
                    </button>
                    <button id="reset" disabled={props.isRunning} onClick={props.onReset}>
                        Reset
                    </button>
                </div>
            </div>

            <div id="status" className="status-value" aria-live="polite">
                {props.status}
            </div>
        </section>
    );
}
