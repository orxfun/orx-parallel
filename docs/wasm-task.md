# Current State

Currently we can use orx-parallel in two ways:
* via rayon-core and wasm-bindgen-rayon. This is demonstrated by examples/wasm_demo_tsp.
* via pool/wasm_web2.rs. This is demonstrated by examples/wasm_demo_tsp2

As far as I could observe the prior is faster than the latter. The goal is to make them have similar performances.

## Step 1

To observe performance, we want to create examples/wasm_perf.rs example which will compare variants:

1. rayon: rayon and wasm-bindgen-rayon (no orx-parallel)
2. rayon-orx: orx-parallel, rayon-core and wasm-bindgen-rayon, as in wasm_demo_tsp
3. orx: just orx-parallel, as in wasm_demo_tsp2.
4. orx3: just orx-parallel with wasm_web3 backend.

We would like to solve a similar TSP problem with all of them and compare their WASM performances.

### Step 1 Implementation

Implemented at `examples/wasm_perf/` as a browser benchmark example with shared TSP logic:

1. rayon: `examples/wasm_perf/crate_rayon`
2. rayon-orx: `examples/wasm_perf/crate_rayon_orx`
3. orx: `examples/wasm_perf/crate_orx`
4. orx3: `examples/wasm_perf/crate_orx3`

Shared algorithm/data source:

- `examples/wasm_perf/tsp_core` (deterministic locations + same 2-opt search implementation)

Browser harness:

- `examples/wasm_perf/web` (manual benchmark UI and structured output)

### Run 4-Variant Benchmark

From `examples/wasm_perf/web`:

```bash
npm install
npm run build:wasm
npm run dev:full
```

Then open the local Vite URL, set benchmark parameters (threads/cities/iterations/warmups/runs), and click **Run**.

The harness executes all four variants sequentially in this order:

1. `rayon`
2. `rayon-orx`
3. `orx`
4. `orx3`

Note: thread count is fixed after first runtime initialization in a page session. To benchmark another thread count (for example 4 then 8), reload the page between runs.

### Step 1 Benchmark Protocol

- Runtime target: Browser (Chrome/Firefox)
- Metrics: wall-clock computation time (ms) and throughput (iterations/s)
- Fairness rules:
	- same algorithm and data across variants
	- warmup runs excluded from measured statistics
	- fixed thread count per session
	- initialization excluded from timing
- Cases:
	- cities: 50, 75
	- iterations: 1000, 10000
	- warmups: 2
	- measured runs: 5
	- threads: run separate sessions for 4 and 8

### Step 1 Results Log

Thread count = 4

```json
WASM benchmark report

threads: 4
cityCounts: 50, 75
iterationCounts: 1000, 10000
warmups: 2
runs: 5
seed: 42

variant | cities | iterations | median_ms | mean_ms | median_ips | mean_ips
--- | ---: | ---: | ---: | ---: | ---: | ---:
rayon | 50 | 1000 | 291.00 | 292.00 | 3436 | 3425
rayon | 50 | 10000 | 3155.00 | 3140.00 | 3170 | 3185
rayon | 75 | 1000 | 1594.00 | 1597.80 | 627 | 626
rayon | 75 | 10000 | 98425.00 | 90954.40 | 102 | 110
rayon-orx | 50 | 1000 | 588.00 | 578.60 | 1701 | 1728
rayon-orx | 50 | 10000 | 5698.00 | 5713.40 | 1755 | 1750
rayon-orx | 75 | 1000 | 2347.00 | 2188.80 | 426 | 457
rayon-orx | 75 | 10000 | 22932.00 | 22702.60 | 436 | 440
orx | 50 | 1000 | 577.00 | 580.60 | 1733 | 1722
orx | 50 | 10000 | 5786.00 | 5784.80 | 1728 | 1729
orx | 75 | 1000 | 2352.00 | 2361.00 | 425 | 424
orx | 75 | 10000 | 23574.00 | 23565.40 | 424 | 424
orx3 | 50 | 1000 | 570.00 | 573.60 | 1754 | 1743
orx3 | 50 | 10000 | 5725.00 | 5710.60 | 1747 | 1751
orx3 | 75 | 1000 | 2362.00 | 2361.40 | 423 | 423
orx3 | 75 | 10000 | 23454.00 | 23432.00 | 426 | 427

Raw JSON:
{
  "config": {
    "threads": 4,
    "cityCounts": [
      50,
      75
    ],
    "iterationCounts": [
      1000,
      10000
    ],
    "warmups": 2,
    "runs": 5,
    "seed": "42"
  },
  "rows": [
    {
      "variant": "rayon",
      "threads": 4,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 291,
      "meanMs": 292,
      "minMs": 290,
      "maxMs": 296,
      "medianIps": 3436.4261168384883,
      "meanIps": 3424.657534246576,
      "samplesMs": [
        296,
        291,
        290,
        290,
        293
      ]
    },
    {
      "variant": "rayon",
      "threads": 4,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 3155,
      "meanMs": 3140,
      "minMs": 2995,
      "maxMs": 3248,
      "medianIps": 3169.572107765452,
      "meanIps": 3184.7133757961783,
      "samplesMs": [
        3079,
        2995,
        3155,
        3223,
        3248
      ]
    },
    {
      "variant": "rayon",
      "threads": 4,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1594,
      "meanMs": 1597.8,
      "minMs": 1470,
      "maxMs": 1668,
      "medianIps": 627.3525721455458,
      "meanIps": 625.860558267618,
      "samplesMs": [
        1663,
        1594,
        1668,
        1594,
        1470
      ]
    },
    {
      "variant": "rayon",
      "threads": 4,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 98425,
      "meanMs": 90954.4,
      "minMs": 55885,
      "maxMs": 103913,
      "medianIps": 101.6002032004064,
      "meanIps": 109.94520331066997,
      "samplesMs": [
        55885,
        101429,
        103913,
        98425,
        95120
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 4,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 588,
      "meanMs": 578.6,
      "minMs": 526,
      "maxMs": 599,
      "medianIps": 1700.6802721088436,
      "meanIps": 1728.3097131005877,
      "samplesMs": [
        599,
        526,
        588,
        594,
        586
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 4,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 5698,
      "meanMs": 5713.4,
      "minMs": 5571,
      "maxMs": 5841,
      "medianIps": 1755.0017550017549,
      "meanIps": 1750.2712920502677,
      "samplesMs": [
        5841,
        5685,
        5571,
        5698,
        5772
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 4,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 2347,
      "meanMs": 2188.8,
      "minMs": 1569,
      "maxMs": 2364,
      "medianIps": 426.075841499787,
      "meanIps": 456.87134502923976,
      "samplesMs": [
        2314,
        2347,
        2364,
        1569,
        2350
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 4,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 22932,
      "meanMs": 22702.6,
      "minMs": 21519,
      "maxMs": 23348,
      "medianIps": 436.0718646432932,
      "meanIps": 440.47818311559035,
      "samplesMs": [
        22478,
        21519,
        23236,
        23348,
        22932
      ]
    },
    {
      "variant": "orx",
      "threads": 4,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 577,
      "meanMs": 580.6,
      "minMs": 570,
      "maxMs": 596,
      "medianIps": 1733.1022530329292,
      "meanIps": 1722.356183258698,
      "samplesMs": [
        596,
        570,
        577,
        585,
        575
      ]
    },
    {
      "variant": "orx",
      "threads": 4,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 5786,
      "meanMs": 5784.8,
      "minMs": 5744,
      "maxMs": 5813,
      "medianIps": 1728.3097131005877,
      "meanIps": 1728.668233992532,
      "samplesMs": [
        5744,
        5786,
        5803,
        5778,
        5813
      ]
    },
    {
      "variant": "orx",
      "threads": 4,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 2352,
      "meanMs": 2361,
      "minMs": 2342,
      "maxMs": 2393,
      "medianIps": 425.1700680272109,
      "meanIps": 423.54934349851754,
      "samplesMs": [
        2393,
        2348,
        2352,
        2370,
        2342
      ]
    },
    {
      "variant": "orx",
      "threads": 4,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 23574,
      "meanMs": 23565.4,
      "minMs": 23334,
      "maxMs": 23679,
      "medianIps": 424.1961482989734,
      "meanIps": 424.3509552140002,
      "samplesMs": [
        23334,
        23574,
        23674,
        23679,
        23566
      ]
    },
    {
      "variant": "orx3",
      "threads": 4,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 570,
      "meanMs": 573.6,
      "minMs": 567,
      "maxMs": 587,
      "medianIps": 1754.3859649122808,
      "meanIps": 1743.3751743375174,
      "samplesMs": [
        587,
        568,
        576,
        570,
        567
      ]
    },
    {
      "variant": "orx3",
      "threads": 4,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 5725,
      "meanMs": 5710.6,
      "minMs": 5609,
      "maxMs": 5768,
      "medianIps": 1746.7248908296945,
      "meanIps": 1751.1294785136413,
      "samplesMs": [
        5768,
        5609,
        5725,
        5703,
        5748
      ]
    },
    {
      "variant": "orx3",
      "threads": 4,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 2362,
      "meanMs": 2361.4,
      "minMs": 2342,
      "maxMs": 2384,
      "medianIps": 423.3700254022015,
      "meanIps": 423.4775980350639,
      "samplesMs": [
        2371,
        2362,
        2384,
        2348,
        2342
      ]
    },
    {
      "variant": "orx3",
      "threads": 4,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 23454,
      "meanMs": 23432,
      "minMs": 23251,
      "maxMs": 23581,
      "medianIps": 426.3665046473949,
      "meanIps": 426.76681461249575,
      "samplesMs": [
        23454,
        23581,
        23402,
        23472,
        23251
      ]
    }
  ]
}
```

Thread count = 8

```json
WASM benchmark report

threads: 8
cityCounts: 50, 75
iterationCounts: 1000, 10000
warmups: 2
runs: 5
seed: 42

variant | cities | iterations | median_ms | mean_ms | median_ips | mean_ips
--- | ---: | ---: | ---: | ---: | ---: | ---:
rayon | 50 | 1000 | 438.00 | 441.00 | 2283 | 2268
rayon | 50 | 10000 | 4329.00 | 4348.80 | 2310 | 2299
rayon | 75 | 1000 | 1768.00 | 1766.40 | 566 | 566
rayon | 75 | 10000 | 91846.00 | 85371.60 | 109 | 117
rayon-orx | 50 | 1000 | 287.00 | 286.20 | 3484 | 3494
rayon-orx | 50 | 10000 | 2810.00 | 2745.40 | 3559 | 3642
rayon-orx | 75 | 1000 | 1152.00 | 1152.20 | 868 | 868
rayon-orx | 75 | 10000 | 11393.00 | 11384.60 | 878 | 878
orx | 50 | 1000 | 284.00 | 282.40 | 3521 | 3541
orx | 50 | 10000 | 2816.00 | 2804.60 | 3551 | 3566
orx | 75 | 1000 | 1139.00 | 1145.40 | 878 | 873
orx | 75 | 10000 | 11326.00 | 11333.00 | 883 | 882
orx3 | 50 | 1000 | 286.00 | 291.80 | 3497 | 3427
orx3 | 50 | 10000 | 2170.00 | 2052.00 | 4608 | 4873
orx3 | 75 | 1000 | 1056.00 | 1061.40 | 947 | 942
orx3 | 75 | 10000 | 10921.00 | 11005.00 | 916 | 909

Raw JSON:
{
  "config": {
    "threads": 8,
    "cityCounts": [
      50,
      75
    ],
    "iterationCounts": [
      1000,
      10000
    ],
    "warmups": 2,
    "runs": 5,
    "seed": "42"
  },
  "rows": [
    {
      "variant": "rayon",
      "threads": 8,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 438,
      "meanMs": 441,
      "minMs": 426,
      "maxMs": 465,
      "medianIps": 2283.10502283105,
      "meanIps": 2267.573696145125,
      "samplesMs": [
        465,
        440,
        438,
        426,
        436
      ]
    },
    {
      "variant": "rayon",
      "threads": 8,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 4329,
      "meanMs": 4348.8,
      "minMs": 4294,
      "maxMs": 4431,
      "medianIps": 2310.00231000231,
      "meanIps": 2299.484915378955,
      "samplesMs": [
        4431,
        4320,
        4329,
        4294,
        4370
      ]
    },
    {
      "variant": "rayon",
      "threads": 8,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1768,
      "meanMs": 1766.4,
      "minMs": 1750,
      "maxMs": 1785,
      "medianIps": 565.6108597285067,
      "meanIps": 566.123188405797,
      "samplesMs": [
        1785,
        1768,
        1768,
        1761,
        1750
      ]
    },
    {
      "variant": "rayon",
      "threads": 8,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 91846,
      "meanMs": 85371.6,
      "minMs": 50280,
      "maxMs": 98281,
      "medianIps": 108.87790431809768,
      "meanIps": 117.13497228586556,
      "samplesMs": [
        98281,
        91846,
        50280,
        95069,
        91382
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 8,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 287,
      "meanMs": 286.2,
      "minMs": 282,
      "maxMs": 288,
      "medianIps": 3484.3205574912895,
      "meanIps": 3494.0600978336824,
      "samplesMs": [
        288,
        282,
        286,
        288,
        287
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 8,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 2810,
      "meanMs": 2745.4,
      "minMs": 2631,
      "maxMs": 2835,
      "medianIps": 3558.7188612099644,
      "meanIps": 3642.4564726451517,
      "samplesMs": [
        2631,
        2631,
        2835,
        2820,
        2810
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 8,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1152,
      "meanMs": 1152.2,
      "minMs": 1132,
      "maxMs": 1171,
      "medianIps": 868.0555555555557,
      "meanIps": 867.9048776254122,
      "samplesMs": [
        1138,
        1152,
        1171,
        1168,
        1132
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 8,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 11393,
      "meanMs": 11384.6,
      "minMs": 11301,
      "maxMs": 11440,
      "medianIps": 877.7319406653207,
      "meanIps": 878.379565377791,
      "samplesMs": [
        11301,
        11367,
        11422,
        11393,
        11440
      ]
    },
    {
      "variant": "orx",
      "threads": 8,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 284,
      "meanMs": 282.4,
      "minMs": 276,
      "maxMs": 286,
      "medianIps": 3521.1267605633807,
      "meanIps": 3541.076487252125,
      "samplesMs": [
        284,
        286,
        285,
        281,
        276
      ]
    },
    {
      "variant": "orx",
      "threads": 8,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 2816,
      "meanMs": 2804.6,
      "minMs": 2761,
      "maxMs": 2832,
      "medianIps": 3551.136363636364,
      "meanIps": 3565.570847892748,
      "samplesMs": [
        2794,
        2816,
        2832,
        2820,
        2761
      ]
    },
    {
      "variant": "orx",
      "threads": 8,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1139,
      "meanMs": 1145.4,
      "minMs": 1124,
      "maxMs": 1172,
      "medianIps": 877.9631255487269,
      "meanIps": 873.0574471800243,
      "samplesMs": [
        1124,
        1159,
        1172,
        1133,
        1139
      ]
    },
    {
      "variant": "orx",
      "threads": 8,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 11326,
      "meanMs": 11333,
      "minMs": 11293,
      "maxMs": 11375,
      "medianIps": 882.9242450997704,
      "meanIps": 882.3788934968676,
      "samplesMs": [
        11326,
        11375,
        11293,
        11348,
        11323
      ]
    },
    {
      "variant": "orx3",
      "threads": 8,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 286,
      "meanMs": 291.8,
      "minMs": 281,
      "maxMs": 322,
      "medianIps": 3496.503496503497,
      "meanIps": 3427.004797806717,
      "samplesMs": [
        286,
        281,
        286,
        322,
        284
      ]
    },
    {
      "variant": "orx3",
      "threads": 8,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 2170,
      "meanMs": 2052,
      "minMs": 1130,
      "maxMs": 2719,
      "medianIps": 4608.294930875576,
      "meanIps": 4873.294346978558,
      "samplesMs": [
        2719,
        2170,
        1130,
        1578,
        2663
      ]
    },
    {
      "variant": "orx3",
      "threads": 8,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1056,
      "meanMs": 1061.4,
      "minMs": 1011,
      "maxMs": 1112,
      "medianIps": 946.9696969696969,
      "meanIps": 942.1518748822309,
      "samplesMs": [
        1011,
        1112,
        1056,
        1090,
        1038
      ]
    },
    {
      "variant": "orx3",
      "threads": 8,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 10921,
      "meanMs": 11005,
      "minMs": 10483,
      "maxMs": 12168,
      "medianIps": 915.6670634557275,
      "meanIps": 908.6778736937755,
      "samplesMs": [
        10532,
        10483,
        12168,
        10921,
        10921
      ]
    }
  ]
}
```

Thread count = 16

```json
WASM benchmark report

threads: 16
cityCounts: 50, 75
iterationCounts: 1000, 10000
warmups: 2
runs: 5
seed: 42

variant | cities | iterations | median_ms | mean_ms | median_ips | mean_ips
--- | ---: | ---: | ---: | ---: | ---: | ---:
rayon | 50 | 1000 | 553.00 | 557.40 | 1808 | 1794
rayon | 50 | 10000 | 5387.00 | 5406.00 | 1856 | 1850
rayon | 75 | 1000 | 8880.00 | 7116.40 | 113 | 141
rayon | 75 | 10000 | 66152.00 | 62730.60 | 151 | 159
rayon-orx | 50 | 1000 | 69.00 | 69.40 | 14493 | 14409
rayon-orx | 50 | 10000 | 982.00 | 998.40 | 10183 | 10016
rayon-orx | 75 | 1000 | 503.00 | 484.00 | 1988 | 2066
rayon-orx | 75 | 10000 | 6483.00 | 6519.20 | 1542 | 1534
orx | 50 | 1000 | 247.00 | 282.80 | 4049 | 3536
orx | 50 | 10000 | 2094.00 | 1970.00 | 4776 | 5076
orx | 75 | 1000 | 832.00 | 815.00 | 1202 | 1227
orx | 75 | 10000 | 7791.00 | 8181.80 | 1284 | 1222
orx3 | 50 | 1000 | 144.00 | 146.80 | 6944 | 6812
orx3 | 50 | 10000 | 2947.00 | 3094.20 | 3393 | 3232
orx3 | 75 | 1000 | 1155.00 | 1152.00 | 866 | 868
orx3 | 75 | 10000 | 11634.00 | 11756.60 | 860 | 851

Raw JSON:
{
  "config": {
    "threads": 16,
    "cityCounts": [
      50,
      75
    ],
    "iterationCounts": [
      1000,
      10000
    ],
    "warmups": 2,
    "runs": 5,
    "seed": "42"
  },
  "rows": [
    {
      "variant": "rayon",
      "threads": 16,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 553,
      "meanMs": 557.4,
      "minMs": 547,
      "maxMs": 574,
      "medianIps": 1808.3182640144664,
      "meanIps": 1794.0437746681018,
      "samplesMs": [
        574,
        547,
        560,
        553,
        553
      ]
    },
    {
      "variant": "rayon",
      "threads": 16,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 5387,
      "meanMs": 5406,
      "minMs": 5307,
      "maxMs": 5559,
      "medianIps": 1856.3207722294414,
      "meanIps": 1849.7965223825381,
      "samplesMs": [
        5403,
        5387,
        5559,
        5374,
        5307
      ]
    },
    {
      "variant": "rayon",
      "threads": 16,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 8880,
      "meanMs": 7116.4,
      "minMs": 2883,
      "maxMs": 10162,
      "medianIps": 112.61261261261261,
      "meanIps": 140.52048788713395,
      "samplesMs": [
        10162,
        4746,
        2883,
        8880,
        8911
      ]
    },
    {
      "variant": "rayon",
      "threads": 16,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 66152,
      "meanMs": 62730.6,
      "minMs": 22193,
      "maxMs": 103443,
      "medianIps": 151.16700931188777,
      "meanIps": 159.41183409691604,
      "samplesMs": [
        97805,
        103443,
        66152,
        24060,
        22193
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 16,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 69,
      "meanMs": 69.4,
      "minMs": 68,
      "maxMs": 72,
      "medianIps": 14492.753623188404,
      "meanIps": 14409.22190201729,
      "samplesMs": [
        70,
        69,
        68,
        68,
        72
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 16,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 982,
      "meanMs": 998.4,
      "minMs": 952,
      "maxMs": 1090,
      "medianIps": 10183.299389002037,
      "meanIps": 10016.02564102564,
      "samplesMs": [
        982,
        957,
        952,
        1011,
        1090
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 16,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 503,
      "meanMs": 484,
      "minMs": 409,
      "maxMs": 527,
      "medianIps": 1988.0715705765408,
      "meanIps": 2066.115702479339,
      "samplesMs": [
        527,
        409,
        461,
        520,
        503
      ]
    },
    {
      "variant": "rayon-orx",
      "threads": 16,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 6483,
      "meanMs": 6519.2,
      "minMs": 5861,
      "maxMs": 7007,
      "medianIps": 1542.4957581366652,
      "meanIps": 1533.9305436249847,
      "samplesMs": [
        6828,
        5861,
        6417,
        6483,
        7007
      ]
    },
    {
      "variant": "orx",
      "threads": 16,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 247,
      "meanMs": 282.8,
      "minMs": 163,
      "maxMs": 528,
      "medianIps": 4048.582995951417,
      "meanIps": 3536.0678925035363,
      "samplesMs": [
        247,
        163,
        175,
        301,
        528
      ]
    },
    {
      "variant": "orx",
      "threads": 16,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 2094,
      "meanMs": 1970,
      "minMs": 1576,
      "maxMs": 2317,
      "medianIps": 4775.549188156639,
      "meanIps": 5076.1421319796955,
      "samplesMs": [
        1698,
        2165,
        1576,
        2317,
        2094
      ]
    },
    {
      "variant": "orx",
      "threads": 16,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 832,
      "meanMs": 815,
      "minMs": 616,
      "maxMs": 1019,
      "medianIps": 1201.923076923077,
      "meanIps": 1226.993865030675,
      "samplesMs": [
        616,
        986,
        622,
        832,
        1019
      ]
    },
    {
      "variant": "orx",
      "threads": 16,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 7791,
      "meanMs": 8181.8,
      "minMs": 7087,
      "maxMs": 9769,
      "medianIps": 1283.532280836863,
      "meanIps": 1222.2249382776406,
      "samplesMs": [
        7626,
        7087,
        7791,
        9769,
        8636
      ]
    },
    {
      "variant": "orx3",
      "threads": 16,
      "cities": 50,
      "iterations": 1000,
      "medianMs": 144,
      "meanMs": 146.8,
      "minMs": 138,
      "maxMs": 160,
      "medianIps": 6944.444444444445,
      "meanIps": 6811.989100817438,
      "samplesMs": [
        144,
        138,
        142,
        150,
        160
      ]
    },
    {
      "variant": "orx3",
      "threads": 16,
      "cities": 50,
      "iterations": 10000,
      "medianMs": 2947,
      "meanMs": 3094.2,
      "minMs": 2713,
      "maxMs": 3775,
      "medianIps": 3393.2813030200205,
      "meanIps": 3231.85314459311,
      "samplesMs": [
        3105,
        2713,
        2931,
        2947,
        3775
      ]
    },
    {
      "variant": "orx3",
      "threads": 16,
      "cities": 75,
      "iterations": 1000,
      "medianMs": 1155,
      "meanMs": 1152,
      "minMs": 1105,
      "maxMs": 1176,
      "medianIps": 865.8008658008658,
      "meanIps": 868.0555555555557,
      "samplesMs": [
        1155,
        1105,
        1176,
        1172,
        1152
      ]
    },
    {
      "variant": "orx3",
      "threads": 16,
      "cities": 75,
      "iterations": 10000,
      "medianMs": 11634,
      "meanMs": 11756.6,
      "minMs": 11564,
      "maxMs": 12275,
      "medianIps": 859.5495960116898,
      "meanIps": 850.586053791062,
      "samplesMs": [
        11605,
        11564,
        11705,
        12275,
        11634
      ]
    }
  ]
}
```

## Step 2

Improve performance of "orx" to make it comparable with options "rayon" and / or "rayon-orx:.
