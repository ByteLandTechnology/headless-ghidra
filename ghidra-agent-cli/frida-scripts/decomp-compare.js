// decomp-compare.js - Compare decompiler output with runtime behavior
// Usage: frida -f <binary> -l decomp-compare.js -- <args>
// Hooks a function and compares its runtime behavior against its decompiled C form

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

console.log("# decomp-compare output");
console.log("version: 1");
console.log("target_module: " + targetModule.name);
console.log("---");

const targetFunc = '%%FUNC%%';
const logFile = '%%DECOMP_LOG%%';

if (!targetFunc) {
    console.error('Usage: --func <name_or_addr>');
    throw new Error('No function specified');
}

// Build export map
var exportMap = {};
targetModule.enumerateExports().forEach(function(e) {
    if (e.type === 'function') {
        exportMap[e.name] = e.address;
    }
});

let funcAddr = exportMap[targetFunc];

if (!funcAddr && /^[0-9a-f]+$/i.test(targetFunc)) {
    funcAddr = ptr(parseInt(targetFunc, 16));
}

if (!funcAddr) {
    console.error('Could not find: ' + targetFunc);
    throw new Error('Function not found');
}

console.error('decomp-compare: monitoring ' + targetFunc + ' at ' + funcAddr);

console.log(JSON.stringify({
    type: 'monitoring',
    function: targetFunc,
    address: funcAddr.toString()
}));

Interceptor.attach(funcAddr, {
    onEnter: function(args) {
        this._args = [];
        for (let i = 0; i < 4; i++) {
            try {
                this._args.push(args[i].toString());
            } catch (e) {
                this._args.push("<error>");
            }
        }
        this._start = Date.now();
    },
    onLeave: function(retval) {
        const elapsed = Date.now() - this._start;
        const record = {
            type: 'call',
            function: targetFunc,
            addr: funcAddr.toString(),
            args: this._args,
            return_value: retval.toString(),
            elapsed_ms: elapsed,
            timestamp: new Date().toISOString()
        };
        console.log(JSON.stringify(record));

        // Infer signature from observed values
        let inferred_ret = 'unknown';
        if (retval.toString().startsWith('0x') && !retval.isNull()) inferred_ret = 'pointer';
        else if (retval.toInt32) inferred_ret = retval.toInt32() === 0 ? 'null' : 'int';

        let inferred_args = this._args.map((a, i) => {
            if (a.startsWith('0x')) return 'pointer arg' + i;
            if (a === '<error>') return 'unknown arg' + i;
            return 'int arg' + i;
        }).join(', ');

        console.log(JSON.stringify({
            type: 'inference',
            function: targetFunc,
            inferred_signature: inferred_ret + ' ' + targetFunc + '(' + inferred_args + ')',
            confidence: 'low'
        }));
    }
});
