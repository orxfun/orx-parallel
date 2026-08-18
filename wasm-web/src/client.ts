import type { WorkerRequest, WorkerResponse } from "./protocol.js";

export type ComputationMap = Record<string, (...args: any[]) => any>;

export type ParallelWorkerOptions<T extends ComputationMap> = {
    bindingsUrl: string | URL;
    methods: readonly (keyof T & string)[];
    threads?: number;
    workerFactory?: () => Worker;
};

export type CancellablePromise<T> = Promise<T> & { cancel: () => void };

type QueueEntry<T> = {
    id: number;
    method: string;
    args: unknown[];
    resolve: (value: T) => void;
    reject: (reason: unknown) => void;
    cancelled: boolean;
};

export class ParallelWorker<T extends ComputationMap> {
    private readonly bindingsUrl: string;
    private readonly methods: Set<string>;
    private readonly threads: number;
    private readonly workerFactory: () => Worker;
    private worker: Worker;
    private queue: QueueEntry<unknown>[] = [];
    private active: QueueEntry<unknown> | undefined;
    private nextId = 1;
    private initialization: Promise<void> | undefined;
    private initializationResolve: (() => void) | undefined;
    private initializationReject: ((reason: unknown) => void) | undefined;
    private terminated = false;
    private initializedThreadCount: number | undefined;

    constructor(options: ParallelWorkerOptions<T>) {
        if (!Number.isInteger(options.threads ?? 0) || (options.threads ?? 0) < 0) {
            throw new Error("threads must be a non-negative integer");
        }
        if (options.methods.length === 0) {
            throw new Error("at least one computation method is required");
        }

        this.bindingsUrl = String(options.bindingsUrl);
        this.methods = new Set(options.methods);
        this.threads = options.threads ?? 0;
        this.workerFactory = options.workerFactory ?? (() => new Worker(new URL("./worker.js", import.meta.url), { type: "module" }));
        this.worker = this.createWorker();
    }

    ready(): Promise<void> {
        if (this.terminated) return Promise.reject(new Error("worker is terminated"));
        if (this.initialization !== undefined) return this.initialization;

        this.initialization = new Promise<void>((resolve, reject) => {
            this.initializationResolve = resolve;
            this.initializationReject = reject;
        });
        this.worker.postMessage({
            type: "init",
            bindingsUrl: this.bindingsUrl,
            threads: this.threads,
            methods: [...this.methods]
        } satisfies WorkerRequest);
        return this.initialization;
    }

    get initializedThreads(): number | undefined {
        return this.initializedThreadCount;
    }

    call<K extends keyof T & string>(method: K, args: Parameters<T[K]>): CancellablePromise<Awaited<ReturnType<T[K]>>> {
        if (this.terminated) return this.rejectedPromise("worker is terminated");
        if (!this.methods.has(method)) return this.rejectedPromise(`method is not allowed: ${method}`);

        let cancelEntry: (() => void) | undefined;
        const promise = new Promise<Awaited<ReturnType<T[K]>>>((resolve, reject) => {
            const entry: QueueEntry<Awaited<ReturnType<T[K]>>> = {
                id: this.nextId++,
                method,
                args,
                resolve,
                reject,
                cancelled: false
            };
            cancelEntry = () => {
                if (entry.cancelled || entry === this.active) return;
                entry.cancelled = true;
                reject(new Error("call cancelled"));
                this.pump();
            };
            this.queue.push(entry as QueueEntry<unknown>);
            void this.ready().then(() => this.pump(), reject);
        });
        const cancellable = promise as CancellablePromise<Awaited<ReturnType<T[K]>>>;
        cancellable.cancel = () => cancelEntry?.();
        return cancellable;
    }

    async restart(): Promise<void> {
        if (this.terminated) throw new Error("worker is terminated");
        this.failPending(new Error("worker restarted"));
        this.worker.terminate();
        this.worker = this.createWorker();
        this.initialization = undefined;
        this.initializedThreadCount = undefined;
        await this.ready();
    }

    terminate(): void {
        if (this.terminated) return;
        this.terminated = true;
        this.failPending(new Error("worker terminated"));
        this.worker.terminate();
    }

    private createWorker(): Worker {
        const worker = this.workerFactory();
        worker.addEventListener("message", (event: MessageEvent<WorkerResponse>) => this.handleResponse(event.data));
        worker.addEventListener("error", (event) => this.failPending(new Error(event.message || "worker failed")));
        return worker;
    }

    private handleResponse(response: WorkerResponse): void {
        if (response.type === "ready") {
            this.initializedThreadCount = response.threads;
            this.initializationResolve?.();
            this.initializationResolve = undefined;
            this.initializationReject = undefined;
            this.pump();
            return;
        }
        if (response.type === "error") {
            if (response.id === undefined) {
                this.initializationReject?.(new Error(response.message));
                this.failPending(new Error(response.message));
                return;
            }
            if (this.active?.id === response.id) {
                this.active.reject(new Error(response.message));
                this.active = undefined;
                this.pump();
            }
            return;
        }
        if (this.active?.id !== response.id) return;
        this.active.resolve(response.value);
        this.active = undefined;
        this.pump();
    }

    private pump(): void {
        if (this.active !== undefined || this.queue.length === 0 || this.terminated) return;
        const entry = this.queue.shift();
        if (entry === undefined) return;
        if (entry.cancelled) {
            this.pump();
            return;
        }
        this.active = entry;
        this.worker.postMessage({ type: "call", id: entry.id, method: entry.method, args: entry.args } satisfies WorkerRequest);
    }

    private failPending(error: Error): void {
        this.initializationReject?.(error);
        this.initializationResolve = undefined;
        this.initializationReject = undefined;
        this.active?.reject(error);
        this.active = undefined;
        for (const entry of this.queue) entry.reject(error);
        this.queue = [];
    }

    private rejectedPromise<T>(message: string): CancellablePromise<T> {
        const promise = Promise.reject<T>(new Error(message)) as CancellablePromise<T>;
        promise.cancel = () => undefined;
        return promise;
    }
}

export type { WorkerRequest, WorkerResponse } from "./protocol.js";