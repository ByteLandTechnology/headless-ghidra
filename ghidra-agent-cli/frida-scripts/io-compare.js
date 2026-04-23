// io-compare.js - Compare captured I/O between original and reconstructed
// Usage: frida -f <binary> -l io-compare.js -- <args>
// Compares two IOLOG files and reports mismatches

{
  const originalLog = JSON.parse(
    new TextDecoder().decode(
      readFileSync(env.IOLOG_ORIG || "/tmp/iolog_original.json"),
    ),
  );
  const reconstructedLog = JSON.parse(
    new TextDecoder().decode(
      readFileSync(env.IOLOG_RECON || "/tmp/iolog_reconstructed.json"),
    ),
  );

  let matches = 0;
  let mismatches = 0;

  for (
    let i = 0;
    i < Math.min(originalLog.length, reconstructedLog.length);
    i++
  ) {
    const orig = originalLog[i];
    const recon = reconstructedLog[i];

    if (orig.type !== recon.type || orig.name !== recon.name) {
      console.log(
        JSON.stringify({
          match: false,
          index: i,
          reason: "type/name mismatch",
          orig,
          recon,
        }),
      );
      mismatches++;
      continue;
    }

    // Compare args
    let argsMatch = orig.args.length === recon.args.length;
    if (argsMatch) {
      for (let j = 0; j < orig.args.length; j++) {
        if (orig.args[j] !== recon.args[j]) {
          argsMatch = false;
          break;
        }
      }
    }

    let retMatch = orig.return_value === recon.return_value;

    if (argsMatch && retMatch) {
      matches++;
    } else {
      console.log(
        JSON.stringify({
          match: false,
          index: i,
          args_match: argsMatch,
          ret_match: retMatch,
          orig_args: orig.args,
          recon_args: recon.args,
          orig_ret: orig.return_value,
          recon_ret: recon.return_value,
        }),
      );
      mismatches++;
    }
  }

  console.log(
    JSON.stringify({
      summary: true,
      matches,
      mismatches,
      total: originalLog.length,
    }),
  );
}
