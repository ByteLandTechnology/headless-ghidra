// signature-analysis.js - Analyze runtime function signatures
// Usage: frida -f <binary> -l signature-analysis.js -- <args>
// Hooks specified functions and infers signatures from observed call sites

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

console.log("# signature-analysis output");
console.log("version: 1");
console.log("target_module: " + targetModule.name);
console.log("target_base: " + targetModule.base);
console.log("---");

const TARGET_FUNCS = '%%FUNCS%%'.split(',').filter(Boolean);
if (TARGET_FUNCS.length === 0 || (TARGET_FUNCS.length === 1 && TARGET_FUNCS[0] === '')) {
    console.error('Usage: --funcs func1,func2,...');
    throw new Error('No functions specified');
}

// Build a map of export name to address
var exportMap = {};
targetModule.enumerateExports().forEach(function(e) {
    if (e.type === 'function') {
        exportMap[e.name] = e.address;
    }
});

const sigDB = {};

for (const funcName of TARGET_FUNCS) {
    let addr = exportMap[funcName];

    if (!addr && /^[0-9a-f]+$/i.test(funcName)) {
        addr = ptr(parseInt(funcName, 16));
    }

    if (!addr) {
        console.error('Could not find: ' + funcName);
        continue;
    }

    const info = { argCount: 0, argTypes: [], retType: null, samples: [] };

    Interceptor.attach(addr, {
        onEnter: function(args) {
            this._args = [];
            for (let i = 0; i < 4; i++) {
                try {
                    const arg = args[i];
                    let typeHint = 'unknown';
                    if (arg.toString().startsWith('0x')) typeHint = 'pointer';
                    else if (arg.toInt32 && arg.toInt32() > 0 && arg.toInt32() < 0x10000) typeHint = 'int';
                    this._args.push(typeHint);
                } catch (e) {
                    this._args.push('error');
                }
            }
            this._start = Date.now();
        },
        onLeave: function(retval) {
            info.argTypes = this._args;
            info.argCount = Math.max(info.argCount, this._args.length);
            let retHint = 'unknown';
            if (retval.toString().startsWith('0x') && !retval.isNull()) retHint = 'pointer';
            else if (retval.toInt32) retHint = retval.toInt32() === 0 ? 'null' : 'int';
            info.retType = retHint;
            info.samples.push({
                args: this._args,
                ret: retval.toString(),
                elapsed_ms: Date.now() - this._start
            });
            console.log(JSON.stringify({
                func: funcName,
                inferred_signature: (info.retType || '?') + ' ' + funcName + '(' + info.argTypes.map((t, i) => t + ' arg' + i).join(', ') + ')',
                arg_count: info.argCount,
                samples: info.samples.length,
                last_call: { args: this._args, ret: retval.toString() }
            }));
        }
    });

    sigDB[funcName] = info;
    console.error('signature-analysis: monitoring ' + funcName + ' at ' + addr);
}

console.log('# Monitoring started. Press Ctrl+C to stop.');
