'use strict';

// ---- Configuration ----
const BRANCH = "v4-enhanced-benches";
const BASE_RESULT_URL = `https://raw.githubusercontent.com/orxfun/orx-parallel/${BRANCH}/benches/results/`;
const BASE_CODE_URL = `https://github.com/orxfun/orx-parallel/blob/${BRANCH}/benches/`;

const CATALOG = {
    early_exit: ['suspicious_find_any', 'suspicious_first'],
    fallible: ['validation_option', 'validation_result'],
    first: ['f', 'ff', 'fff', 'i', 'id', 'lf', 'mf', 'mfmf', 'mi'],
    het: ['advanced', 'simple'],
    recursive: ['tree_traversal'],
    reduce: ['f', 'id', 'l', 'm', 'mf', 'mfm', 'mfmf'],
    stateful_using: ['monte_carlo_batch'],
    throughput_linear: ['log_collect', 'log_reduce'],
    xap: ['cf', 'cmmmm', 'f', 'ff', 'fff', 'ffff', 'i', 'ii', 'iii', 'iiii',
        'l_cons', 'l_iter', 'l_vec', 'lf', 'll_cons', 'll_iter', 'll_vec',
        'lll_cons', 'lll_iter', 'lll_vec', 'lm', 'm', 'mf', 'mfm', 'mfmf',
        'mfmfmf', 'mm', 'mmmm'],
};

// Columns to skip when building filters (non-filterable metadata + target value)
const SKIP_COLS = new Set(['t', 'i', 'a', 'time (ns)']);
const TIME_COL = 'time (ns)';

// ---- State ----
const state = {
    category: null,
    bench: null,
    headers: [],
    rows: [],
    filterCols: [],
    filters: {},      // { colName: Set<string> }
};

let chartInstance = null;

// ---- Utilities ----
function escHtml(str) {
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

function parseCsv(text) {
    const lines = text.trim().split('\n');
    if (lines.length < 2) return { headers: [], rows: [] };
    const headers = lines[0].split(',').map(h => h.trim());
    const rows = lines.slice(1).map(line => {
        const vals = line.split(',');
        const obj = {};
        headers.forEach((h, i) => { obj[h] = (vals[i] ?? '').trim(); });
        return obj;
    });
    return { headers, rows };
}

/**
 * Returns sorted method names: seq first, then grouped by prefix (rayon, orx, orx-fixed),
 * sorted numerically within each group, then anything else alphabetically.
 */
function sortMethods(methods) {
    const order = (m) => {
        if (m === 'seq') return [0, 0, m];
        const rayonMatch = m.match(/^rayon-(\d+)$/);
        if (rayonMatch) return [1, parseInt(rayonMatch[1]), m];
        const orxMatch = m.match(/^orx-(\d+)$/);
        if (orxMatch) return [2, parseInt(orxMatch[1]), m];
        const orxFixedMatch = m.match(/^orx-fixed-(\d+)$/);
        if (orxFixedMatch) return [3, parseInt(orxFixedMatch[1]), m];
        return [4, 0, m];
    };
    return [...methods].sort((a, b) => {
        const [ka, na, sa] = order(a);
        const [kb, nb, sb] = order(b);
        if (ka !== kb) return ka - kb;
        if (na !== nb) return na - nb;
        return sa.localeCompare(sb);
    });
}

function sortMethodsForChart(methods) {
    return [...methods].sort((a, b) => {
        if (a === 'seq') return -1;
        if (b === 'seq') return 1;
        return a.localeCompare(b);
    });
}

/** Assign a consistent color to a method name. */
function methodColor(method) {
    if (method === 'seq') return '#47476b';
    const hash = (() => {
        let value = 0;
        for (const c of method) value = (value * 31 + c.charCodeAt(0)) & 0xffff;
        return value;
    })();
    const tone = (hue, saturation, minLightness, maxLightness) => {
        const span = maxLightness - minLightness;
        const lightness = minLightness + (hash % 1000) / 1000 * span;
        return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
    };

    if (/^rayon(?:[-_].+)?$/.test(method)) {
        return tone(210, 82, 34, 64);
    }
    if (/^orx-fixed(?:[-_].+)?$/.test(method)) {
        return tone(28, 95, 46, 72);
    }
    if (/^orx(?:[-_].+)?$/.test(method) || /^xap(?:[-_].+)?$/.test(method)) {
        return tone(116, 70, 42, 72);
    }
    // Generic fallback: hash to a palette
    const palette = ['#6366f1', '#ec4899', '#14b8a6', '#f97316', '#8b5cf6', '#0ea5e9'];
    return palette[hash % palette.length];
}

// ---- DOM helpers ----
function showPanel(id, visible) {
    document.getElementById(id).style.display = visible ? '' : 'none';
}

function setEmptyMsg(msg, visible = true) {
    document.getElementById('empty-msg').textContent = msg;
    showPanel('empty-panel', visible);
}

// ---- Category & Benchmark lists ----
function renderCategories() {
    const ul = document.getElementById('category-list');
    ul.innerHTML = '';
    for (const cat of Object.keys(CATALOG)) {
        const li = document.createElement('li');
        const btn = document.createElement('button');
        btn.textContent = cat;
        btn.className = cat === state.category ? 'active' : '';
        btn.addEventListener('click', () => selectCategory(cat));
        li.appendChild(btn);
        ul.appendChild(li);
    }
}

function renderBenches() {
    const ul = document.getElementById('bench-list');
    ul.innerHTML = '';
    if (!state.category) return;
    for (const bench of CATALOG[state.category]) {
        const li = document.createElement('li');
        const btn = document.createElement('button');
        btn.textContent = bench;
        btn.className = bench === state.bench ? 'active' : '';
        btn.addEventListener('click', () => selectBench(bench));
        li.appendChild(btn);
        ul.appendChild(li);
    }
}

function selectCategory(cat) {
    state.category = cat;
    state.bench = null;
    renderCategories();
    renderBenches();
    // Auto-select the first benchmark in the new category
    const benches = CATALOG[cat];
    if (benches && benches.length > 0) {
        selectBench(benches[0]);
    }
}

function selectBench(bench) {
    state.bench = bench;
    renderBenches();
    loadBench(state.category, bench);
}

// ---- CSV loading & parsing ----
async function loadBench(category, bench) {
    // Hide data panels, show loading message
    setEmptyMsg('Loading…');
    showPanel('link-panel', false);
    showPanel('filters-panel', false);
    showPanel('chart-panel', false);
    showPanel('table-panel', false);

    const resultUrl = `${BASE_RESULT_URL}${category}/${category}_${bench}.csv`;
    const codeUrl = `${BASE_CODE_URL}${category}/${bench}.rs`;

    try {
        const resp = await fetch(resultUrl);
        if (!resp.ok) throw new Error(`HTTP ${resp.status} for ${resultUrl}`);
        const text = await resp.text();
        const { headers, rows } = parseCsv(text);

        state.headers = headers;
        state.rows = rows;
        state.filterCols = headers.filter(h => !SKIP_COLS.has(h));

        // Initialize all filters with all distinct values selected
        state.filters = {};
        for (const col of state.filterCols) {
            const vals = [...new Set(rows.map(r => r[col]))];
            state.filters[col] = new Set(vals);
        }

        setEmptyMsg('', false);
        renderLink(resultUrl, codeUrl);
        renderFilters();
        renderChart();
        renderTable();
        showPanel('link-panel', true);
        showPanel('filters-panel', true);
        showPanel('chart-panel', true);
        showPanel('table-panel', true);
    } catch (err) {
        console.error(err);
        setEmptyMsg(`Failed to load results: ${err.message}`);
        showPanel('filters-panel', false);
        showPanel('chart-panel', false);
        showPanel('table-panel', false);
    }
}

// ---- Link ----
function renderLink(resultUrl, codeUrl) {
    const resultLink = document.getElementById('result-link');
    resultLink.innerHTML = `<a href="${resultUrl}" target="_blank">CSV</a>`

    const codeLink = document.getElementById('code-link');
    codeLink.innerHTML = `<a href="${codeUrl}" target="_blank">Code</a>`
}


// ---- Filters ----
function renderFilters() {
    const group = document.getElementById('filter-group');
    group.innerHTML = '';

    for (const col of state.filterCols) {
        const allVals = [...new Set(state.rows.map(r => r[col]))];
        const sorted = col === 'method' ? sortMethods(allVals) : allVals.slice().sort();

        const row = document.createElement('div');
        row.className = 'filter-row';

        const label = document.createElement('span');
        label.className = 'filter-label';
        label.textContent = col;
        row.appendChild(label);

        const chips = document.createElement('div');
        chips.className = 'filter-chips';

        for (const val of sorted) {
            const chip = document.createElement('button');
            chip.className = 'chip' + (state.filters[col].has(val) ? ' active' : '');
            chip.textContent = val;
            chip.addEventListener('click', (event) => toggleFilter(col, val, chip, event));
            chips.appendChild(chip);
        }

        row.appendChild(chips);
        group.appendChild(row);
    }
}

function toggleFilter(col, val, chip, event) {
    const sel = state.filters[col];
    if (event.ctrlKey) {
        if (sel.has(val)) {
            sel.delete(val);
            chip.classList.remove('active');
        } else {
            sel.add(val);
            chip.classList.add('active');
        }
    } else {
        sel.clear();
        sel.add(val);

        const chips = chip.parentElement.querySelectorAll('.chip');
        for (const otherChip of chips) {
            otherChip.classList.toggle('active', otherChip === chip);
        }
    }
    renderChart();
    renderTable();
}

function getFilteredRows() {
    return state.rows.filter(row =>
        state.filterCols.every(col => state.filters[col].has(row[col]))
    );
}

// ---- Chart ----
function renderChart() {
    const filtered = getFilteredRows();

    // Determine selected methods from the method filter (or all distinct if method isn't a filter col)
    const methodSel = state.filters['method'];
    const methods = methodSel
        ? sortMethodsForChart([...methodSel])
        : sortMethodsForChart([...new Set(filtered.map(r => r['method']))]);

    // For each selected method, average time over filtered rows where method == that method
    const sums = Object.fromEntries(methods.map(m => [m, 0]));
    const counts = Object.fromEntries(methods.map(m => [m, 0]));

    for (const row of filtered) {
        const m = row['method'];
        if (Object.prototype.hasOwnProperty.call(sums, m)) {
            const t = parseFloat(row[TIME_COL]);
            if (!isNaN(t)) {
                sums[m] += t;
                counts[m]++;
            }
        }
    }

    const labels = methods;
    const data = methods.map(m => counts[m] > 0 ? Math.round(sums[m] / counts[m]) : 0);
    const colors = methods.map(m => methodColor(m));

    // Destroy previous chart and recreate (avoids stale data across benchmark switches)
    if (chartInstance) {
        chartInstance.destroy();
        chartInstance = null;
    }

    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const gridColor = prefersDark ? 'rgba(148,163,184,0.12)' : 'rgba(20,33,61,0.08)';
    const tickColor = prefersDark ? '#94a3b8' : '#51607a';

    const ctx = document.getElementById('bench-chart').getContext('2d');
    chartInstance = new Chart(ctx, {
        type: 'bar',
        data: {
            labels,
            datasets: [{
                label: 'Average time (ns)',
                data,
                backgroundColor: colors,
                borderColor: colors,
                borderWidth: 1,
                borderRadius: 6,
                borderSkipped: false,
            }],
        },
        options: {
            responsive: true,
            plugins: {
                legend: { display: false },
                tooltip: {
                    callbacks: {
                        label: ctx => `${ctx.formattedValue} ns`,
                    },
                },
            },
            scales: {
                x: {
                    ticks: { color: tickColor, font: { family: '"IBM Plex Sans", sans-serif', size: 12 } },
                    grid: { color: gridColor },
                },
                y: {
                    beginAtZero: true,
                    ticks: { color: tickColor, font: { family: '"IBM Plex Sans", sans-serif', size: 12 } },
                    grid: { color: gridColor },
                    title: {
                        display: true,
                        text: 'Time (ns)',
                        color: tickColor,
                        font: { family: '"IBM Plex Sans", sans-serif', size: 12 },
                    },
                },
            },
        },
    });
}

// ---- Table ----
function renderTable() {
    const filtered = getFilteredRows();
    const container = document.getElementById('table-container');

    if (filtered.length === 0) {
        container.innerHTML = '<div class="state-msg">No rows match the current filters.</div>';
        return;
    }

    let html = '<table><thead><tr>';
    for (const h of state.headers) {
        if (h === 'time (ns)') {
            html += `<th style="text-align:right;">${escHtml(h)}</th>`;
        } else {
            html += `<th>${escHtml(h)}</th>`;
        }
    }
    html += '</tr></thead><tbody>';
    for (const row of filtered) {
        html += '<tr>';
        for (const h of state.headers) {
            if (h === 'time (ns)') {
                html += `<td style="text-align:right;">${escHtml(parseInt(row[h]).toLocaleString('en') ?? '')}</td>`;
            } else {
                html += `<td>${escHtml(row[h] ?? '')}</td>`;
            }
        }
        html += '</tr>';
    }
    html += '</tbody></table>';
    container.innerHTML = html;
}

// ---- Init ----
renderCategories();
renderBenches();