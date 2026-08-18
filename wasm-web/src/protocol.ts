export type WorkerRequest =
    | {
        type: "init";
        bindingsUrl: string;
        threads: number;
        methods: string[];
    }
    | {
        type: "call";
        id: number;
        method: string;
        args: unknown[];
    };

export type WorkerResponse =
    | { type: "ready"; threads: number }
    | { type: "result"; id: number; value: unknown }
    | { type: "error"; id?: number; message: string };

export type WasmBindings = {
    default: (options?: unknown) => Promise<unknown>;
    init_parallel_runtime: (threads: number) => Promise<unknown>;
    [method: string]: unknown;
};