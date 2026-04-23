// call-tree-trace.js - Trace call tree at runtime
// Usage: frida -f <binary> -l call-tree-trace.js -- <args>
// Outputs a JSONL trace of all function calls with stack depth

const maxDepth = parseInt("%%MAX_DEPTH%%" || "32") || 32;
const callLog = [];
let depth = 0;

// Global trace that intercepts all function entries
const libPaths = "%%LIBS%%"; // placeholder for library paths

function instrumentLib(libPath) {
  const mod = Module.load(libPath);
  console.error("Instrumenting: " + libPath);
  mod.enumerateSymbols().forEach((sym) => {
    if (sym.type === "function") {
      try {
        const addr = sym.address;
        Interceptor.attach(addr, {
          onEnter: function (args) {
            depth++;
            if (depth <= maxDepth) {
              callLog.push({
                type: "call",
                func: sym.name,
                lib: libPath,
                depth: depth,
                timestamp: Date.now(),
              });
            }
          },
          onLeave: function (retval) {
            if (depth <= maxDepth) {
              callLog.push({
                type: "ret",
                func: sym.name,
                depth: depth,
                timestamp: Date.now(),
              });
            }
            depth--;
            if (callLog.length >= 1000) {
              console.log(
                JSON.stringify({ type: "batch", calls: callLog.splice(0) }),
              );
            }
          },
        });
      } catch (e) {
        // Skip unhookable symbols
      }
    }
  });
}

// Instrument current module
var targetModule = null;
Process.enumerateModules().forEach(function (m) {
  if (
    m.name !== "frida" &&
    !m.name.startsWith("libclang") &&
    !m.name.startsWith("libswift")
  ) {
    if (!m.path.startsWith("/System") && !m.path.startsWith("/usr/lib")) {
      targetModule = m;
    }
  }
});

if (targetModule) {
  console.log("# call-tree-trace output");
  console.log("version: 1");
  console.log("target_module: " + targetModule.name);
  console.log("---");
  instrumentLib(targetModule.path);
}

console.error("call-tree-trace: active, max_depth=" + maxDepth);
