// frida-trace.js - Trace specific functions
// Usage: frida -l frida-trace.js -f <executable> -- [func1,func2,...]

var targetModule = null;

console.log("# frida-trace output");
console.log("version: 1");

// Find target module
Process.enumerateModules().forEach(function(m) {
    if (m.name !== "frida" && !m.name.startsWith("libclang") && !m.name.startsWith("libswift")) {
        if (!m.path.startsWith("/System") && !m.path.startsWith("/usr/lib")) {
            targetModule = m;
        }
    }
});

if (targetModule) {
    console.log("target_module: " + targetModule.name);
    console.log("target_base: " + targetModule.base);
} else {
    console.log("target_module: null");
}

console.log("---");
console.log("trace:");

// Helper to safely get argument
function getArg(args, idx) {
    try {
        return args[idx].toString();
    } catch (e) {
        return "<error>";
    }
}

// Trace all exports
if (targetModule) {
    targetModule.enumerateExports().forEach(function(e) {
        if (e.type === "function") {
            (function(exportName, exportAddr) {
                Interceptor.attach(exportAddr, {
                    onEnter: function(args) {
                        var caller = this.returnAddress;
                        var callerMod = null;
                        try {
                            callerMod = Process.findModuleByAddress(caller);
                        } catch (e) {}
                        console.log(JSON.stringify({
                            type: "call",
                            function: exportName,
                            caller: (callerMod ? callerMod.name : "unknown"),
                            args: [getArg(args, 0), getArg(args, 1), getArg(args, 2), getArg(args, 3)]
                        }));
                    },
                    onLeave: function(retval) {
                        console.log(JSON.stringify({
                            type: "return",
                            function: exportName,
                            return_value: retval.toString()
                        }));
                    }
                });
            })(e.name, e.address);
        }
    });
}
