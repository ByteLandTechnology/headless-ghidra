// frida-run.js - Run executable with tracing
// Usage: frida -l frida-run.js -f <executable> -- [args...]

var callLog = [];
var startTime = Date.now();
var targetModule = null;

console.log("# frida-run output");
console.log("version: 1");

// Find target module
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
  console.log("target_module: " + targetModule.name);
  console.log("target_base: " + targetModule.base);
} else {
  console.log("target_module: null");
}

console.log("---");
console.log("calls:");

// Trace all exports
if (targetModule) {
  targetModule.enumerateExports().forEach(function (e) {
    if (e.type === "function") {
      (function (exportName, exportAddr) {
        Interceptor.attach(exportAddr, {
          onEnter: function (args) {
            var caller = this.returnAddress;
            var callerMod = null;
            try {
              callerMod = Process.findModuleByAddress(caller);
            } catch (e) {}
            var argStrs = [];
            for (var i = 0; i < 4; i++) {
              try {
                argStrs.push(args[i].toString());
              } catch (e) {
                argStrs.push("<error>");
              }
            }
            console.log(
              JSON.stringify({
                type: "call",
                function: exportName,
                caller: callerMod ? callerMod.name : "unknown",
                args: argStrs,
              }),
            );
          },
          onLeave: function (retval) {
            console.log(
              JSON.stringify({
                type: "return",
                function: exportName,
                return_value: retval.toString(),
              }),
            );
          },
        });
      })(e.name, e.address);
    }
  });
}
