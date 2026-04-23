// dispatch-vtable-trace.js - Trace vtable dispatch at runtime
// Usage: frida -f <binary> -l dispatch-vtable-trace.js -- <args>
// Monitors virtual method calls by hooking vtable entries

{
  const VTABLE_SUSPECT_RANGES = JSON.parse(env.VTABLE_RANGES || "[]");
  // e.g. [{base: '0x1000', count: 8, stride: 8}]

  const traces = [];

  function hookVtable(baseAddr, count, stride) {
    console.error(
      "Hooking vtable at " +
        baseAddr +
        ", " +
        count +
        " entries, stride=" +
        stride,
    );
    for (let i = 0; i < count; i++) {
      const entryAddr = baseAddr.add(i * stride);
      try {
        const methodPtr = entryAddr.readPointer();
        if (methodPtr && !methodPtr.isNull()) {
          const off = i;
          Interceptor.attach(methodPtr, {
            onEnter: function (args) {
              traces.push({
                type: "vcall",
                vtable_base: baseAddr.toString(),
                method_index: off,
                method_addr: methodPtr.toString(),
                args: Array.from(args).map((a) => a.toString()),
                timestamp: Date.now(),
              });
            },
          });
        }
      } catch (e) {
        // Failed to read vtable entry
      }
    }
  }

  if (VTABLE_SUSPECT_RANGES.length > 0) {
    VTABLE_SUSPECT_RANGES.forEach((r) => {
      hookVtable(ptr(r.base), r.count, r.stride || Process.pointerSize);
    });
  } else {
    // Auto-detect: scan for vtable-like patterns in memory
    console.error(
      "dispatch-vtable-trace: no ranges specified, use VTABLE_RANGES env var",
    );
  }

  // Dump traces periodically
  setInterval(() => {
    if (traces.length > 0) {
      console.log(JSON.stringify({ batch: traces.splice(0) }));
    }
  }, 5000);
}
