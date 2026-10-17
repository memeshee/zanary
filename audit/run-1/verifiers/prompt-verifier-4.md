# Candidate verifier prompt — verifier-4 on `crates/report/src/receipt.rs:hex_decode+short:utf8-byte-slice-of-untrusted-receipt-fields`

You did not write this candidate. Try to refute it from repository source and bounded
local evidence. Do not contact deployed endpoints or external/shared services. Run
target-controlled code only inside the approved OS-enforced sandbox: no external
network, empty allowlisted environment, read-only target and tools, scratch-only
writes, and explicit low resource and wall-clock limits. If any control is unavailable,
do not execute; retain the exact missing capability as a needs_validation blocker.
Treat every scratch entry as target-controlled after execution. After the sandbox and
all its processes terminate, only trusted parent-side code may promote a predeclared
scratch-relative file, following the promotion procedure block included verbatim in
this prompt. You and target code never write retained artifacts. If promotion is
unavailable or fails, do not use that file as evidence.

1. Verify every trace and evidence file, positive line number, scope, and description.
   Confirm the first entry is a real lower-trust entrypoint and the last is the
   claimed sink or boundary effect.
2. Reconstruct the strongest source-visible validation, identity, authorization,
   normalization, lifecycle, framework, and containment controls on the path.
   Where the architecture summary names a comparable baseline, note whether it
   shares the pattern — as calibration, never as grounds to dismiss.
3. For a proposed confirmed candidate, independently reproduce the minimum observed
   result when possible. Verify inputs, interface shape, conditions, and affected
   dummy principal/resource. Do not infer a stronger result or continue after it.
4. Verify that likelihood, impact, confidence, and the proposed source fix match only
   what the evidence establishes.
5. For a proposed needs_validation candidate, decide whether the blocker is genuinely
   outside source/local observation. If source refutes the trace, reject it. If the
   missing fact remains decisive, keep needs_validation and make the local and
   owner-observed plans exact and non-destructive.
6. Preserve the fingerprint for the same source-derived root cause across every state.

Return exactly one JSON object and no surrounding prose:
{"decision": "confirmed|needs_validation|rejected", "record": { ... }}
where record exactly matches the decision's verdict branch of the schema included
in this prompt. A corrected record replaces the hunter's wording.

## The candidate you must try to refute

Hunter: `hunter-receipt` · proposed verdict: `confirmed` · fingerprint: `crates/report/src/receipt.rs:hex_decode+short:utf8-byte-slice-of-untrusted-receipt-fields`

```json
{
  "proposed_verdict": "confirmed",
  "fingerprint": "crates/report/src/receipt.rs:hex_decode+short:utf8-byte-slice-of-untrusted-receipt-fields",
  "title": "verify-receipt aborts with a Rust panic (exit 101, no verdict) on a receipt whose hex fields or attestation key are not ASCII",
  "description": "`zanary-report verify-receipt` exists so a stranger can check a receipt somebody else supplied: main.rs:104-106 reads the path from argv and deserializes it into `zanary_observation::Receipt`, whose `signer`, `signature` and `attestation.{format,receipt_key,wallet_ak,signature}` fields are arbitrary UTF-8 `String`s (crates/observation/src/lib.rs:115-141), and receipt.rs then slices those strings by BYTE offset in five places. `hex_decode` (receipt.rs:352-363) only checks that the byte length is even before `u8::from_str_radix(&hex[i..i + 2], 16)`, and `short` (receipt.rs:188-194) only checks `hex.len() > 20` before `&hex[..8]` / `&hex[hex.len() - 8..]`; slicing a `&str` panics when the index falls inside a multi-byte character. A receipt whose `signer` is `a\u20ac` (4 bytes, even) aborts the verifier before any verdict line is printed; a receipt whose signed fields are perfectly valid and which carries a grafted `attestation` object with a non-ASCII `receipt_key` passes `signature : VALID` and both `claim ok` lines and then panics inside `short()` on the no-`--fvk` branch (main.rs:184-190) - reachable because the attestation is not part of the signed bytes, as the crate's own test asserts (receipt.rs:487-504). The tool's contract is an exit code a caller or workflow reads (0 verified, 1 FAIL, 2 input error; main.rs:49-55, 135-138, 199-201); a panic is none of those, so for that artifact the caller gets no verdict at all.",
  "root_cause": "Byte-offset slicing of caller-controlled strings guarded only by a byte length: `hex_decode` slices `&hex[i..i + 2]` after `hex.len().is_multiple_of(2)` (receipt.rs:359) and `short` slices `&hex[..8]` / `&hex[hex.len() - 8..]` after `hex.len() > 20` (receipt.rs:190-191). Neither calls `is_char_boundary` or rejects non-ASCII, so any multi-byte UTF-8 character straddling the index panics with \"end byte index N is not a char boundary\".",
  "intended_behavior": "Malformed receipt fields are reported, not fatal: the existing error paths return `hex string has an odd length`, `the signer key is N bytes, expected 32`, `the signer key is not a valid Orchard spend-authorizing key`, the CLI prints `signature : INVALID` / `claim : INVALID` and exits 1, and `--fvk` mismatches exit 1 (main.rs:117-197; docs/FLOW-RUNNER-NOTES.md:343-355). Every parseable receipt file should produce a verdict with exit 0 (pass), 1 (FAIL) or 2 (input error, e.g. an undecodable UFVK) - never a process abort.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/report/src/main.rs",
      "line": 104,
      "scope": "verify_receipt",
      "description": "reads the caller-given receipt path and deserializes the file as zanary_observation::Receipt; signer/signature/attestation fields are unvalidated Strings"
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/main.rs",
      "line": 117,
      "scope": "verify_receipt",
      "description": "calls zanary_report::receipt::verify_signature(&receipt) on that untrusted value"
    },
    {
      "kind": "sink",
      "file": "crates/report/src/receipt.rs",
      "line": 359,
      "scope": "hex_decode",
      "description": "u8::from_str_radix(&hex[i..i + 2], 16) slices the caller-supplied signer/signature/wallet_ak string by byte index after only an even-length check -> panic"
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/main.rs",
      "line": 188,
      "scope": "verify_receipt",
      "description": "with no --fvk, a receipt that passed both checks and carries an attestation has its unvalidated receipt_key/wallet_ak echoed through short()"
    },
    {
      "kind": "sink",
      "file": "crates/report/src/receipt.rs",
      "line": 190,
      "scope": "short",
      "description": "&hex[..8] (and &hex[hex.len() - 8..] on line 191) slices the caller-supplied key by byte index after only a length check -> panic"
    }
  ],
  "evidence": [
    {
      "file": "crates/report/src/receipt.rs",
      "line": 353,
      "description": "hex_decode's only guard on caller-supplied input is `hex.len().is_multiple_of(2)`"
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 359,
      "description": "the byte-offset slice `&hex[i..i + 2]` that panics: \"end byte index 2 is not a char boundary; it is inside '\u20ac' (bytes 1..4 of string)\""
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 190,
      "description": "short()'s `&hex[..8]` slice that panics: \"end byte index 8 is not a char boundary; it is inside '\u20ac' (bytes 7..10 of string)\""
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 191,
      "description": "short()'s symmetric `&hex[hex.len() - 8..]` slice, equally unchecked"
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 65,
      "description": "first sink on the verdict path: hex_decode(signer_hex), reached before any other check in verify_signature; three more call sites at receipt.rs:72, 157, 165"
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 105,
      "description": "the receipt is read from a caller-given path and parsed with serde_json, so every field is attacker-chosen text (main.rs:104-106)"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 115,
      "description": "signer: Option<String> - no charset validation (signature at :118, attestation at :124, attestation.signature at :140, attestation.receipt_key/wallet_ak at :136/:138)"
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 489,
      "description": "the crate's own test `the_attestation_is_not_part_of_the_receipt_signature` proves the attestation is outside the signed bytes, so a bogus attestation can be grafted onto a valid receipt"
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 184,
      "description": "the no---fvk branch that echoes an unvalidated attestation through short(), i.e. the second sink is reached with only a validly signed receipt"
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 145,
      "description": "verify_attestation's mismatch messages call short() on the caller-supplied receipt_key and signer, the same unchecked slice under --fvk"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "A receipt file with a multi-byte UTF-8 character at a non-char-boundary byte index in `signer` (variant A; no valid signature required - the panic precedes every check), or in `signature`/`attestation.wallet_ak`/`attestation.signature`; variant B (the short() sink) additionally needs the receipt to pass verify_signature and validate_claim and to carry an `attestation` object whose `receipt_key` holds such a value."
    },
    {
      "kind": "system_configuration",
      "description": "No flags are needed: variant A panics on the first line of the verdict path, variant B on the no---fvk attestation branch. With `--fvk`, the same unchecked short() slices are reached from verify_attestation's mismatch messages (receipt.rs:145-154) for the same class of input."
    }
  ],
  "execution": {
    "attacker_perspective": "Any party whose receipt a verifier inspects - which is the tool's entire purpose - can make `zanary-report verify-receipt` die instead of returning a verdict by putting one non-ASCII character in a hex field. Variant A needs no key and no valid signature; variant B leaves the receipt's signature and claims valid and only grafts an `attestation` object, which the receipt signature does not cover.",
    "payloads": [
      "{\"label\":\"hunter probe\",\"format\":\"zanary-receipt-v1\",\"body\":\"{\\\"amount_zat\\\":1234567,\\\"invoice_id\\\":\\\"4471\\\",\\\"memo_sha256\\\":\\\"60016cf4bf951820ab397c737584734efac9c519a1f8964e8e8d749c62c50748\\\",\\\"txid\\\":\\\"44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\\\"}\",\"claims\":[\"invoice 4471 was paid 1234567 zatoshis in transaction 44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\"],\"discloses\":[],\"signer\":\"a\\u20ac\",\"signature\":\"00\"}",
      "{\"label\":\"hunter probe\",\"format\":\"zanary-receipt-v1\",\"body\":\"{\\\"amount_zat\\\":1234567,\\\"invoice_id\\\":\\\"4471\\\",\\\"memo_sha256\\\":\\\"60016cf4bf951820ab397c737584734efac9c519a1f8964e8e8d749c62c50748\\\",\\\"txid\\\":\\\"44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\\\"}\",\"claims\":[\"invoice 4471 was paid 1234567 zatoshis in transaction 44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\"],\"discloses\":[],\"signer\":\"280f51c8bafa209126d11a661978c5aa40a4bd61f41eb08fed0fc34255ae602e\",\"signature\":\"3b350150293b3639889a41858c8245d1adfb4d96af0987ab685d9d85a82660311af31ac936908d4668bba291880bf56dfb0ab4e533f49584016bac2006d94d1c\",\"attestation\":{\"format\":\"zanary-receipt-attestation-v1\",\"receipt_key\":\"aaaaaaa\\u20acbbbbbbbbbbbbbb\",\"wallet_ak\":\"3bae3a087f1476fe7722d4421baf2027de4cd2390be4d1489bb2162be7bfea1b\",\"signature\":\"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\"}}"
    ],
    "instructions": [
      "Build the CLI in the approved disposable sandbox copy: docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo build --offline -p zanary-report'",
      "Write payload 1 to /t/hunter/payload-a.json (it is the exact file used) and run /t/target/debug/zanary-report verify-receipt /t/hunter/payload-a.json; echo exit=$?",
      "Write payload 2 to /t/hunter/payload-b.json and run /t/target/debug/zanary-report verify-receipt /t/hunter/payload-b.json; echo exit=$?",
      "Unit-level equivalent, also run: append the fixture-generating probe module to the sandbox copy of crates/report/src/receipt.rs and run CARGO_BUILD_JOBS=6 cargo test --offline -p zanary-report receipt::hunter_probe -- --nocapture --test-threads=1 (the module is in check-probe-module.txt)."
    ],
    "observed_result": "Payload 1: the CLI printed receipt/format/claim lines, then `thread 'main' (23) panicked at crates/report/src/receipt.rs:359:36:` / `end byte index 2 is not a char boundary; it is inside '\u20ac' (bytes 1..4 of string)` and `exit=101`; no `signature :` line was ever printed. Payload 2: `signature : VALID`, `signer : 280f51c8...602e`, both `claim ok` lines, then `thread 'main' (30) panicked at crates/report/src/receipt.rs:190:30:` / `end byte index 8 is not a char boundary; it is inside '\u20ac' (bytes 7..10 of string)` and `exit=101`. Control on the same build: the same receipt with an ASCII signer prints `summary: signature verified offline (no node, no indexer, no rig)` and exits 0. Unit level: probe_a and probe_b both recorded the panic (caught and printed), probe_d showed the grafted receipt passes verify_signature and validate_claim."
  },
  "remediation": {
    "strategy": "Reject non-hex input before any slicing and make the shortening helper char-boundary safe, reusing the implementations the repository already contains (crates/report/src/node.rs:193 and crates/probe/src/bin/zanary-seed.rs:714 decode via hex.as_bytes().chunks(2) + std::str::from_utf8, which returns a clean error instead of panicking). Add both payloads as regression fixtures asserting exit 1 with an INVALID verdict rather than a panic.",
    "code_changes": [
      {
        "file_name": "crates/report/src/receipt.rs",
        "fixed_code": "fn hex_decode(hex: &str) -> Result<Vec<u8>> {\n    let bytes = hex.as_bytes();\n    if !bytes.len().is_multiple_of(2) {\n        return Err(anyhow!(\"hex string has an odd length\"));\n    }\n    if !bytes.iter().all(u8::is_ascii_hexdigit) {\n        return Err(anyhow!(\"hex string has a non-hex character (zanary-receipt-v1 says lowercase hex)\"));\n    }\n    (0..bytes.len())\n        .step_by(2)\n        .map(|i| {\n            u8::from_str_radix(&hex[i..i + 2], 16)\n                .map_err(|e| anyhow!(\"invalid hex at byte {}: {e}\", i / 2))\n        })\n        .collect()\n}\n\n/// `first8\u2026last8` without assuming ASCII: never slices inside a character.\nfn at_byte(s: &str, n: usize) -> usize {\n    s.char_indices()\n        .map(|(i, _)| i)\n        .chain(std::iter::once(s.len()))\n        .find(|&i| i >= n)\n        .unwrap_or(s.len())\n}\n\npub fn short(hex: &str) -> String {\n    if hex.len() > 20 {\n        let head = at_byte(hex, 8);\n        let tail_start = at_byte(hex, hex.len() - 8);\n        format!(\"{}\u2026{}\", &hex[..head], &hex[tail_start..])\n    } else {\n        hex.to_string()\n    }\n}"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "medium",
      "reason": "The trigger is trivially cheap for anyone who supplies a receipt - one non-ASCII character in `signer` needs no key and no valid signature - but it only fires for a consumer that runs `zanary-report verify-receipt` on that file, which is exactly the tool's documented use (strangers verifying someone else's receipt) and not something the honest producer emits."
    },
    "impact": {
      "score": "low",
      "reason": "Observed effect is a process abort: a Rust panic message plus exit 101, with no verdict and no FAIL line. There is no false PASS (101 is non-zero, so an exit-0 gate is not fooled), no memory corruption and no code execution; the artifact simply cannot be verified by this tool, and the panic message leaks only a source path and the offending string."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "Four panics were reproduced in the sandbox with the reviewed revision's code (two CLI invocations, two unit probes), each naming the exact slice; the source trace is complete from argv/file read to both sinks, and the two safe in-tree variants of the same helper confirm the intended behaviour."
  }
}
```

## Its linked coverage unit and the hunter's checks

- coverage_id: `crates%2Freport%2Fsrc%2Freceipt.rs%23verify_signature%20%2F%20verify_attestation%20%2F%20validate_claim::a%20flow-signed%20receipt%20%28signer%20ak%20%2B%20optional%20wallet%20attestation%29%20-%3E%20the%20offline%20%60verify-receipt%60%20verdict%3A%20signature%20validity%2C%20attestation-to-wallet%20binding%2C%20claim%2Fbody%20agreement%2C%20exit%20code::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- surface: `crates/report/src/receipt.rs#verify_signature / verify_attestation / validate_claim` (subsystem `published-evidence`)
- boundary: `a flow-signed receipt (signer ak + optional wallet attestation) -> the offline `verify-receipt` verdict: signature validity, attestation-to-wallet binding, claim/body agreement, exit code`
- attack class: `ATTACK-CLASSES.md#Cryptography and secrets` · assignment owner: `hunter-receipt`
- unit reviewed paths: ["README.md", "crates/observation/src/lib.rs", "crates/probe/src/flow.rs", "crates/report/src/main.rs", "crates/report/src/receipt.rs", "docs/FLOW-RUNNER-NOTES.md", "docs/THREAT-MODEL.md", "schemas/zanary-receipt-v1.json"]

The unit's checks, including the local checks whose promoted artifacts you may read
(paths are relative to the run directory `/home/kiter/security-audit-skill/zanary/run-1`):

```json
[
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/report/src/receipt.rs",
      "crates/report/src/main.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "the verdict binds exactly what it claims: the receipt signature covers {format, body, claims, discloses} under the key the receipt NAMES; the attestation is outside those bytes; --signer pins the named key and --fvk binds it to a wallet's ak through the wallet's own attestation signature; claim/body agreement is checked by validate_claim.",
    "method": "source",
    "result": "Source-confirmed for the binding model: signing_payload/attestation_payload cover only those fields (receipt.rs:24-40, 100-115); verify_signature checks the signature under receipt.signer (receipt.rs:55-94), so a bare pass means only 'that key signed this' - documented at receipt.rs:10-12 and printed as `binding : none` (main.rs:191-196); the `signer` field itself is not in the signed payload, but substituting it breaks verification because it is the verification key; verify_attestation compares attestation.receipt_key against receipt.signer and attestation.wallet_ak against the UFVK-derived ak before checking the wallet's own signature (receipt.rs:125-185), and an attestation does not move the receipt's signed bytes (receipt.rs:487-504), so neither replaying nor stripping an attestation yields a --fvk pass. Nothing binds body/claims to a chain, invoice or amount beyond the signer's assertion - the module documents that limitation, so it is not a finding.",
    "artifact": null
  },
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/report/src/receipt.rs",
      "crates/report/src/main.rs"
    ],
    "invariant": "a receipt file that is not ASCII hex in its key/signature fields, or that carries an unvalidated attestation, is reported with a verdict (INVALID, exit 1) or echoed safely - never by aborting the verifier.",
    "method": "local",
    "result": "Violated (candidate 1). Four panics reproduced in the sandbox: unit probes probe_a (receipt.rs:359:36, signer=\"a<euro>\") and probe_b (receipt.rs:190:30, 24-byte key), and CLI runs `verify-receipt /t/hunter/bad-hex-signer.json` -> no verdict printed, exit 101 and `verify-receipt /t/hunter/grafted-attestation.json` -> `signature : VALID` + both `claim ok` lines, then panic, exit 101. Raw unit log in check-probe-log.txt.",
    "artifact": "agents/hunter-receipt/artifacts/check-receipt-panic.txt"
  },
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/report/src/receipt.rs",
      "schemas/zanary-receipt-v1.json"
    ],
    "invariant": "a receipt whose claims state different figures than its body is refused by validate_claim (`claim : INVALID`, exit 1), as the schema promises.",
    "method": "local",
    "result": "Violated (candidate 2). The executed receipt carries body invoice 4471 / 1234567 zatoshis / txid 44f2...337e and the claim `invoice 44710 was paid 12345678 zatoshis in transaction 44f2...337e`; validate_claim returned Ok at unit level and the CLI printed both `claim ok` lines and exited 0. The check is a three-needle `words.contains(&needle)` loop, so any superstring satisfies it.",
    "artifact": "agents/hunter-receipt/artifacts/check-claim-containment.txt"
  },
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/report/src/receipt.rs",
      "schemas/zanary-receipt-v1.json"
    ],
    "invariant": "the implementation is the check the shipped schema promises (same accept/reject set for third-party tools) and the key it accepts as `signer` is one an Orchard wallet could hold.",
    "method": "source",
    "result": "Two divergences, both hardening rather than findings: (a) uppercase hex is accepted for body txid/memo_sha256 (receipt.rs:301 is_ascii_hexdigit) and for signer/signature/attestation fields, while the schema requires ^[0-9a-f]{64}$/^[0-9a-f]{128}$ and the code's own message claims 'zanary-receipt-v1 says lowercase hex' - executed: uppercase-txid fixture exits 0 with `claim ok`; (b) the identity point (32 zero bytes) is accepted as `signer` (executed: r=B, s=1 verifies, `signer : 000...0`), because reddsa 0.5.2 allows the identity for Orchard and pasta_curves decodes the all-zero encoding as it; a real Orchard ak can never be the identity. Neither grants a pass an attacker could not already obtain with their own key.",
    "artifact": null
  },
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/report/src/main.rs",
      "crates/report/src/receipt.rs"
    ],
    "invariant": "the built CLI's verdicts match the documented behaviour table for the controls, for the unattested + --fvk case, for a garbage UFVK, and for a flag given without a value.",
    "method": "local",
    "result": "Controls on the same sandbox build: clean receipt -> `signature : VALID`, `binding : none`, exit 0; clean receipt + `--fvk <local merchant.ufvk>` -> `FAIL - nothing binds this receipt to that wallet`, exit 1; `--fvk notaufvk` -> `error: decoding the unified full viewing key: Address is not Bech32m encoded`, exit 2; `--signer` with no value is silently dropped and still exits 0 (hardening 3); a bad signature exits 1, not the `2` the docs' Known gaps claims (hardening 6).",
    "artifact": "agents/hunter-receipt/artifacts/check-cli-run.txt"
  },
  {
    "agent_id": "hunter-receipt",
    "reviewed_paths": [
      "crates/probe/src/flow.rs",
      "docs/FLOW-RUNNER-NOTES.md",
      "docs/THREAT-MODEL.md",
      "README.md"
    ],
    "invariant": "the producer signs receipts with a dedicated key and fresh randomness, and the documented behaviour/exit-code contract matches what the verifier does.",
    "method": "source",
    "result": "Producer is sound: sign_receipt uses ZIP-32 account `receipt_account` (default 1, refused when equal to merchant_account: flow.rs:73-78, 263-267, 383-386) and signs each payload with rand::rngs::OsRng (flow.rs:1496) - no nonce reuse, no weak randomness, and the printed material is only public keys and 8+8 prefixes, no secrets in errors or logs. The documented table matches the verifier for the rows that still reproduce (clean 0, unattested + --fvk 1, wrong signer 1), but its fixtures are canary-era and untracked and the Known-gaps exit-code sentence is wrong (hardening 5 and 6).",
    "artifact": null
  }
]
```

Read the promoted files under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/` — those are the hunter's retained
local evidence. You may read a hunter's artifact, but every decisive claim must be re-established by you.

## Architecture facts (verbatim, written by the parent)

# Architecture — zanary (Phase 1 recon, run in the parent)

Target: `/home/kiter/hackathon/zanary` · repo `memeshee/zanary` · commit `7058406` · worktree clean at recon.
Profile `standard`, budget 14 agent invocations. Run dir outside the target: `~/security-audit-skill/zanary/run-1`.

## What the system is

A Rust workspace (38 `.rs` files, 14,465 lines, 100 tracked files) that runs a Zcash **regtest** payment
flow on a throwaway chain, observes what the chain/merchant/observer can see, scores it against 13 named
leak properties, hashes the evidence, and publishes claims that expire unless re-proved.

Crates: `probe` (CLI + rig driver + flow runner + bounded payer + CI gate), `properties` (the 13
properties + fixtures), `report` (report/anchor/verify/index/matrix + the renderer for the published page),
`observation` (the observation model, grants, ledger), plus fixtures/tests and `bin/zanary-seed`.

## Entry points and who can reach them

| entry point | reachable by | what it trusts |
|---|---|---|
| `zanary-probe {wallet,address,fund,shield,pay,anchor,flow,test,payer,invoice,verify-invoice}` | whoever runs the binary | CLI args, flow/grant/invoice JSON, env (`ZANARY_LWD_URL`, `ZANARY_UID/GID`, timeouts) |
| `zanary-report {verify,verify-receipt,build,index,matrix,verify-anchor}` | local user, CI | report/run/receipt JSON + a node/indexer URL |
| `harness/up.sh` + `harness/docker-compose.yml` | local user, CI, **any consumer of the composite action** | docker/podman availability, `ZANARY_UID/GID` |
| `action.yml` (composite GitHub Action, `uses: memeshee/zanary@main`) | **any third-party workflow** | action `inputs:` — interpolated into `run:` steps |
| `.github/workflows/privacy-gate.yml` (5 jobs: tests, private, control, payer, publish) | push/PR/dispatch/**weekly cron** | repo contents; publishes Pages + uploads artifacts |
| `scripts/{demo-run.sh,bounded-payer-demo.sh,ci-summary.py}` | local user, CI | env knobs `ZANARY_DEMO_RUNS`, `ZANARY_PAYER_DEMO_DIR`, `ZANARY_LOG` |
| published Pages site (`/`, `/video/`) + uploaded CI artifacts | **the public internet** (public repo, public Pages) | the evidence files it renders/copies |

## Trust boundaries

1. **Third-party CI ↔ this repo's automation.** `action.yml` is designed so a stranger's workflow runs it
   (`uses: memeshee/zanary@main` is the whole setup). Anything it interpolates from `inputs:` into a shell
   is a boundary between that consumer's workflow and their runner.
2. **Public repo/artifacts ↔ the rig's secrets.** Actions artifacts on a public repo are world-downloadable;
   the Pages deploy is world-readable. Wallet material, viewing keys, or RPC credentials reaching either is
   a disclosure boundary crossing.
3. **Untrusted data ↔ the probe CLI.** Flow definitions, invoices, grants and reports are JSON files that a
   user (or a judge, or CI) supplies; `--out`/`--definition`/`--grant` are caller-controlled paths.
4. **Host network ↔ the local rig.** `harness/docker-compose.yml` publishes zcashd JSON-RPC (18232/18233)
   and lightwalletd gRPC/HTTP (9067/9068) on the host.
5. **Authority model ↔ money movement.** The bounded payer decides whether a payment happens, from a grant
   (budget, per-payment cap, receiver allowlist, expiry, kill switch). Its ledger is the audit trail; the
   `payer-policy` property is the detector.
6. **Claim ↔ observer.** The published evidence (receivers, txids, memos, pool totals) is *intended* to be
   public; the tool's promise is that it never publishes more than the claim needs.

## Secrets and sensitive state

- `harness/zcash.conf` carries `rpcuser`/`rpcpassword` for the regtest node — **committed on purpose** and
  documented as throwaway, in a repo whose README tells people to run it locally. Not a secret in the usual
  sense; it *is* the credential that protects the JSON-RPC control plane (boundary 4).
- Wallet DBs, chain data, per-run evidence and keys live under `data/` and are gitignored (`/data`).
- No API keys, tokens, or `.env` in the tree (checked: no tracked file matches `*.env|secret|key|token|pem`).
- `video/zanary-demo.mp4` (3.6 MB) is a committed artifact, deliberately.

## What the tests need

`cargo test --workspace` is offline (fixtures + golden reports). The demo scripts, the flow runner and the
gate need **Docker plus the local regtest node/indexer over loopback** (`ZANARY_LWD_URL=http://127.0.0.1:9067`).
Anything requiring that rig is outside a `--network=none` sandbox and can only ever land as
`needs_validation` unless an offline path exists.

## Sandbox available

The methodology requires OS-enforced sandboxing for target-controlled execution: no external network,
read-only target, scratch-only writes, bounded resources.

Available: Docker with locally-present images (`rust:1-slim`, `node:22-slim`, `rust:1-slim` verified for
`cargo --version`), `--network=none`, a disposable `git archive HEAD` copy at
`/home/kiter/.hermes/cache/scratch/zanary-sandbox`, and the host cargo registry mounted read-only at
`/usr/local/cargo/registry` so `cargo test --offline` needs no network. Smoke-tested before dispatch —
see `hunter-protocol.md`.

**Sandbox proof (run before the hunters' results were collected).** From the disposable
`git archive HEAD` copy with `rust-toolchain.toml` removed (the pin names `channel = "stable"`, which rustup
can only resolve with network — the sandbox correctly refused: `dns error: Temporary failure in name
resolution`, no silent fetch) and the host registry mounted read-only:

```
docker run --rm --network=none -v $SB:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t \
  rust:1-slim cargo test --offline -p zanary-properties --lib
→ test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured
```

So: the unit suite runs offline inside the sandbox, target code executes with no network, and a local check
is available to hunters for this unit class.

Not available: any sandbox that can reach the regtest chain (the rig's containers are shared host state and
out of bounds for hunters), and any image that is not already local (no pulls, by policy).

## Parent-side scans performed (coverage, recorded here)

Run in the parent before wave 1, read-only, over the whole history (`git rev-list --all`) rather than the
working tree only:

| scan | result |
|---|---|
| secret-shaped patterns across every commit (`uview1`, `upriv1`, `secret-extended-key`, `ghp_`, `AKIA`, `AIza`, `BEGIN … PRIVATE KEY`, `lightwalletd.*password`) | **0 hits** |
| files ever added or deleted on secret-shaped paths | only source/docs names (`wallet.rs`, `WALLET-NOTES.md`, `harness/seed.sh`, the pre-rename `canary-seed.rs`) |
| was `data/` (wallet DB, chain data, per-run evidence) ever committed? | **never** — 0 paths in history |
| anything sensitive still tracked today (`wallet`, `*.key`, `seed`, `*.env`, `*.pem`) | only source files whose *names* match; no key material |
| viewing keys inside recorded evidence (`data/runs/demo/demo-private/{report,run}.json`) | 0 hits for `ufvk`/`uview`/`unified_full_viewing` |

One observation belongs to the published-evidence unit rather than here, and the parent is not the unit's
hunter: `docs/WALLET-NOTES.md` commits a dev **seed phrase** and dev **UFVK**. The phrase is the canonical
BIP-39 test vector (`abandon ×11 · about`) and the wallet is a throwaway regtest wallet, so nothing private
is disclosed — no key material of value exists to disclose. If a finding, it is a *hardening* note about the
habit (a doc-level pattern that would be a disclosure if repeated on a wallet with real history), not a
crossed boundary. Recorded here so the report does not have to rediscover it.

## Coverage note

No prior `run-*` directory exists for this target, so this is the first pass: one wave of 5 units, a
post-wave coverage critic and a distinct final-clean critic, and one fresh verifier per candidate. Absence
of findings on a unit means that unit's assignment was covered, never that the repo is clean.

## Companion validation blocks for this unit (verbatim)

### [companion validation block] MEMORY-SAFETY-AND-BINARY.md#Validation rules — verbatim

## Validation rules (apply before reporting ANY finding here)

1. Establish a realistic untrusted entry and exact operation that violates a bounds, type, lifetime, ABI, concurrency, loader, or authority invariant.
2. Classify the observable effect: invalid read, invalid write, stale alias, wrong object, uninitialized output, unauthorized image load, deadlock, or safe process termination. Do not claim a stronger effect than observed.
3. Run the narrowest local harness, existing test, sanitizer, or fuzzer needed to reproduce the effect. Verify sanitizer coverage of the faulting operation and record architecture/build conditions.
4. For concurrency, use a deterministic schedule or sanitizer trace. For binary loading, prove the checked identity differs from the mapped identity and name the lower-trust writer.
5. Return `confirmed` findings only with exact input, source trace, and observed result. Return `needs_validation` for a specific unresolved reachability, ABI, build, deployment, or runtime fact and state the bounded check needed.

## Promotion procedure (verbatim; reference only — the parent performs it, you never do)

```text
Artifact promotion procedure (trusted parent-side code only):
Reference only for you: the parent performs these steps; you never perform them.

Before execution, the parent opens and retains trusted, non-inheritable directory
descriptors for the agent's scratch/ and artifacts/ roots, and records an allowlist
of expected scratch-relative artifact files plus explicit per-file and cumulative
byte limits. Never pass those descriptors to the agent or sandbox. After the sandbox
and all its processes terminate, trusted parent-side code promotes each allowlisted
file separately:

1. Validate the declared relative path: reject absolute, empty, `.`, `..`, or
   symlinked components.
2. Walk each parent component from the retained scratch-root descriptor with
   no-follow directory-relative operations; never reopen by path.
3. Open the leaf no-follow and nonblocking.
4. Verify with `fstat` that it is a regular file with link count exactly one and
   within the recorded per-file and cumulative byte limits.
5. Enforce those limits again while reading from that descriptor.
6. Copy exactly the verified size, repeat `fstat`, and reject a changed identity,
   type, link count, or size.
7. For the destination, walk every parent component from the retained
   artifacts-root descriptor with no-follow directory-relative operations; require
   each existing component to be a real directory, and create any missing directory
   exclusively before reopening and verifying it no-follow.
8. Create the leaf exclusively without following links, verify that the opened
   destination is a regular file with link count exactly one, and copy from the
   verified source descriptor without reopening either path.
9. Use equivalent race-safe APIs on non-POSIX systems.
10. Never recursively copy or glob scratch, extract an archive into artifacts, or
    open or promote a symlink, FIFO, socket, device, directory, hard-linked file,
    changing file, or file that exceeds its bound.
11. If any check is unavailable, cannot be enforced, or fails, discard the scratch
    entry; if it is decisive evidence, retain `needs_validation` with the exact
    promotion blocker.
```

## Source/local execution boundary and the approved sandbox

Target source is read-only: `/home/kiter/hackathon/zanary` (commit `7058406`, worktree clean — do not modify it).
Target-controlled code may run only inside the OS-enforced sandbox. The approved disposable copy of the
reviewed revision is `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD` copy with `rust-toolchain.toml` removed so the image's own
toolchain is used — do not restore the pin). Rebuild it yourself if needed:
`rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p <crate> <filter> 2>&1 | tail -30'
```

No external network, no image pulls, no `sudo`, no host paths except `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and the read-only registry,
writes only inside `/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` and your own scratch. Out of bounds: the regtest rig's containers and host ports
(18232/18233/9067/9068), any network service, any public URL, and anything that mutates state outside your
scratch. If a decisive fact lives behind one of those, it stays a `needs_validation` blocker — name it exactly.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-4/scratch` (scratch-relative files only, `check-*.txt` / `check-*.json` /
`evidence.txt`, 64 KiB per file, 256 KiB total); the parent promotes into `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-4/artifacts`. You never write
retained artifacts, another agent's directory, or any shared run file.

## Schema branches (verbatim from `report-schema.json`)

`confirmed`:

```json
{
  "type": "object",
  "description": "A source-grounded vulnerability that was independently demonstrated.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "confirmed"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$",
      "description": "A stable source-derived identifier that does not change between validation states."
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "intended_behavior": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "conditions": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "authentication_level",
              "authorization_role",
              "user_interaction",
              "system_configuration",
              "network_routing",
              "environmental_dependency",
              "data_state",
              "timing_dependency",
              "third_party_dependency"
            ]
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "execution": {
      "type": "object",
      "description": "Target-neutral reproduction in the target's native interface.",
      "properties": {
        "attacker_perspective": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "payloads": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },
        "instructions": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "observed_result": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "attacker_perspective",
        "payloads",
        "instructions",
        "observed_result"
      ],
      "additionalProperties": false
    },
    "remediation": {
      "type": "object",
      "properties": {
        "strategy": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "code_changes": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "file_name": {
                "type": "string",
                "minLength": 1
              },
              "fixed_code": {
                "type": "string"
              }
            },
            "required": [
              "file_name",
              "fixed_code"
            ],
            "additionalProperties": false
          }
        }
      },
      "required": [
        "strategy"
      ],
      "additionalProperties": false
    },
    "severity": {
      "type": "object",
      "properties": {
        "likelihood": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "impact": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "overall_severity": {
          "type": "string",
          "enum": [
            "informational",
            "low",
            "medium",
            "high",
            "critical"
          ]
        }
      },
      "required": [
        "likelihood",
        "impact",
        "overall_severity"
      ],
      "additionalProperties": false
    },
    "confidence": {
      "type": "object",
      "properties": {
        "score": {
          "type": "string",
          "enum": [
            "low",
            "medium",
            "high"
          ]
        },
        "reason": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "score",
        "reason"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "root_cause",
    "intended_behavior",
    "trace",
    "evidence",
    "conditions",
    "execution",
    "remediation",
    "severity",
    "confidence"
  ],
  "additionalProperties": false
}
```

`needs_validation`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate whose decisive validation is blocked.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "needs_validation"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "blockers": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "string",
        "minLength": 1,
        "visibleContent": true
      }
    },
    "validation_plan": {
      "type": "object",
      "properties": {
        "local": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "deployment": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "blockers",
    "validation_plan"
  ],
  "additionalProperties": false
}
```

`rejected`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate refuted during validation.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "rejected"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "reason": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "reason"
  ],
  "additionalProperties": false
}
```

## Prior records with the same fingerprint

None. This target has no prior audit run, and no other current candidate carries this fingerprint.

## Your result

Return exactly one JSON object, no prose and no code fences:
`{"decision": "confirmed|needs_validation|rejected", "record": { ... }}` where `record` exactly matches
the decision's verdict branch above (`confirmed` records carry `root_cause`, `intended_behavior`,
`conditions`, `execution`, `remediation`, `severity`, `confidence`; `needs_validation` records carry
`claimed_root_cause`, `blockers`, a nonempty `validation_plan` with at least one applicable `local` or
`deployment` step and no severity or execution; `rejected` records carry `claimed_root_cause` and `reason`).
A corrected record replaces the hunter's wording. Keep the fingerprint unchanged unless you found a
genuinely different root cause — and then say so in the record.
