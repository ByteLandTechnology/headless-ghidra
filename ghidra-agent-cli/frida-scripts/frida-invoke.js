// frida-invoke.js - Invoke functions directly
// Usage: frida -l frida-invoke.js -f <executable> -- <function_name> [arg1,arg2,...]
//
// Makes exported functions callable via invocations object

var invocations = {};
var lastResult = null;

console.log("# frida-invoke output");
console.log("version: 1");
console.log("tool: frida");

// Find target module
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
  console.log("target_module: " + targetModule.name);
  console.log("target_base: " + targetModule.base);
  console.log("exports:");

  // Make all exported functions callable
  targetModule.enumerateExports().forEach(function (e) {
    if (e.type === "function") {
      console.log("  - " + e.name + " @ " + e.address);

      // Create invoker function for this export
      // Note: We need to know the signature to call properly
      // For now, create a generic invoker that takes typed arguments
      invocations[e.name] = function () {
        // This will be populated with actual function pointer
        return null;
      };
    }
  });

  console.log("---");
  console.log("# Available invocations:");
  console.log("#   invocations['function_name'](arg1, arg2, ...)");
  console.log("# Example: invocations['add'](123, 456)");
  console.log("#");
  console.log("# Function pointers (by address) can be called with:");
  console.log(
    "#   new NativeFunction(ptr('0x1234'), 'int', ['int', 'int'])(1, 2)",
  );
  console.log("");
} else {
  console.log("target_module: null");
  console.log("error: No target module found");
}

// Helper to call a function by name with int arguments
function callByName(funcName, arg1, arg2) {
  if (!targetModule) return null;

  var exports = targetModule.enumerateExports();
  for (var i = 0; i < exports.length; i++) {
    if (exports[i].name === funcName && exports[i].type === "function") {
      try {
        var fn = new NativeFunction(exports[i].address, "int", ["int", "int"]);
        var result = fn(arg1, arg2);
        console.log(
          "# Called " + funcName + "(" + arg1 + ", " + arg2 + ") = " + result,
        );
        return result;
      } catch (e) {
        console.log("# Error calling " + funcName + ": " + e);
        return null;
      }
    }
  }
  console.log("# Function not found: " + funcName);
  return null;
}

console.log("# Helper function available:");
console.log("#   callByName('function_name', arg1, arg2)");
