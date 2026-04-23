// io-capture.js - Intercept function entry/exit, record arg/return values
// Usage: frida -f <binary> -l io-capture.js -- <args>
// Output: JSON lines to stdout, one per invocation

function hookAddr(addr, name) {
  Interceptor.attach(addr, {
    onEnter: function (args) {
      const record = {
        type: "call",
        name: name,
        timestamp: Date.now(),
        args: [],
      };
      for (let i = 0; i < 4; i++) {
        try {
          record.args.push(args[i].toString());
        } catch (e) {
          record.args.push("<error>");
        }
      }
      this.record = record;
      this.startTime = Date.now();
    },
    onLeave: function (retval) {
      this.record.duration_ms = Date.now() - this.startTime;
      this.record.return_value = retval.toString();
      console.log(JSON.stringify(this.record));
    },
  });
}

// Get target module
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
  console.log("# io-capture output");
  console.log("version: 1");
  console.log("target_module: " + targetModule.name);
  console.log("target_base: " + targetModule.base);
  console.log("---");

  // Trace all exports
  targetModule.enumerateExports().forEach(function (e) {
    if (e.type === "function") {
      hookAddr(e.address, e.name);
    }
  });
} else {
  console.error("No target module found");
}
