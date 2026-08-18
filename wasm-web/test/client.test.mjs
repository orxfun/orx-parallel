import assert from "node:assert/strict";
import test from "node:test";
import { ParallelWorker } from "../dist/client.js";

class FakeWorker {
    listeners = new Map();
    calls = [];
    terminated = false;

    addEventListener(type, listener) {
        const current = this.listeners.get(type) ?? [];
        current.push(listener);
        this.listeners.set(type, current);
    }

    postMessage(message) {
        if (message.type === "init") {
            queueMicrotask(() => this.emit("message", { data: { type: "ready", threads: 4 } }));
            return;
        }

        this.calls.push(message.method);
        setTimeout(() => {
            this.emit("message", {
                data: { type: "result", id: message.id, value: message.args[0] * 2 }
            });
        }, 0);
    }

    terminate() {
        this.terminated = true;
    }

    emit(type, event) {
        for (const listener of this.listeners.get(type) ?? []) listener(event);
    }
}

test("initializes once and serializes calls", async () => {
    const fake = new FakeWorker();
    const worker = new ParallelWorker({
        bindingsUrl: "./bindings.js",
        methods: ["double"],
        workerFactory: () => fake
    });

    const first = worker.call("double", [2]);
    const second = worker.call("double", [3]);

    assert.equal(await first, 4);
    assert.equal(await second, 6);
    assert.equal(worker.initializedThreads, 4);
    assert.deepEqual(fake.calls, ["double", "double"]);
    worker.terminate();
    assert.equal(fake.terminated, true);
});

test("cancels a queued call", async () => {
    const fake = new FakeWorker();
    const worker = new ParallelWorker({
        bindingsUrl: "./bindings.js",
        methods: ["double"],
        workerFactory: () => fake
    });

    const first = worker.call("double", [2]);
    const second = worker.call("double", [3]);
    const secondRejection = assert.rejects(second, /call cancelled/);
    second.cancel();

    assert.equal(await first, 4);
    await secondRejection;
    assert.deepEqual(fake.calls, ["double"]);
    worker.terminate();
});

test("rejects a worker-reported call error", async () => {
    const fake = new FakeWorker();
    fake.postMessage = function (message) {
        if (message.type === "init") {
            queueMicrotask(() => this.emit("message", { data: { type: "ready", threads: 4 } }));
            return;
        }
        queueMicrotask(() => this.emit("message", {
            data: {
                type: "error",
                id: message.id,
                message: "wasm binding double expects 1 arguments but received 2"
            }
        }));
    };

    const worker = new ParallelWorker({
        bindingsUrl: "./bindings.js",
        methods: ["double"],
        workerFactory: () => fake
    });

    await assert.rejects(worker.call("double", [2, 3]), /expects 1 arguments but received 2/);
    worker.terminate();
});