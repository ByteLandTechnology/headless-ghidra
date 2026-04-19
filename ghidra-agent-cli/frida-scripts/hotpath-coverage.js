// hotpath-coverage.js - Measure hot path coverage during execution
// Usage: frida -f <binary> -l hotpath-coverage.js -- <args>
// Tracks which basic blocks are executed and reports coverage stats

const hotBlocks = {}; // addr -> count
const HOT_THRESHOLD = parseInt('%%HOT_THRESHOLD%%' || '100') || 100;
const REPORT_INTERVAL = parseInt('%%REPORT_INTERVAL_MS%%' || '10000') || 10000;

// Find target module
var targetModule = null;
Process.enumerateModules().forEach(function(m) {
    if (m.name !== "frida" && !m.name.startsWith("libclang") && !m.name.startsWith("libswift")) {
        if (!m.path.startsWith("/System") && !m.path.startsWith("/usr/lib")) {
            targetModule = m;
        }
    }
});

if (!targetModule) {
    console.error("No target module found");
    throw new Error("No target module");
}

console.log("# hotpath-coverage output");
console.log("version: 1");
console.log("target_module: " + targetModule.name);
console.log("---");
console.error('hotpath-coverage: instrumenting ' + targetModule.name);

console.log(JSON.stringify({
    type: 'started',
    threshold: HOT_THRESHOLD,
    report_interval_ms: REPORT_INTERVAL
}));

// Use Stalker for coverage tracking
Stalker.follow({
    events: {
        call: true,
        ret: true
    },
    onReceive: function(events) {
        const parser = Stalker.parse(events);
        parser.forEach(item => {
            if (item[0] === 'call') {
                const addr = item[1].toString();
                hotBlocks[addr] = (hotBlocks[addr] || 0) + 1;
            }
        });
    }
});

// Periodic hot path report
setInterval(() => {
    const hot = Object.entries(hotBlocks)
        .filter(([, count]) => count >= HOT_THRESHOLD)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 50)
        .map(([addr, count]) => ({ addr, count }));

    if (hot.length > 0) {
        console.log(JSON.stringify({
            type: 'hotpath_report',
            threshold: HOT_THRESHOLD,
            hot_blocks: hot,
            total_blocks: Object.keys(hotBlocks).length,
            timestamp: new Date().toISOString()
        }));
    }
}, REPORT_INTERVAL);
