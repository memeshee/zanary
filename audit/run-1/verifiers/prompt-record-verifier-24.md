# Phase-5 record verifier — verifier-24 on `crates/report/src/receipt.rs:validate_claim:claims-body-agreement-is-substring-containment`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`confirmed`)

```json
{
  "verdict": "confirmed",
  "fingerprint": "crates/report/src/receipt.rs:validate_claim:claims-body-agreement-is-substring-containment",
  "title": "verify-receipt certifies claim/body agreement for a receipt whose claims state different figures (substring containment)",
  "description": "`validate_claim` is documented as the machine-readable half of \"a stranger can check it\": receipt.rs:250-256 says the rules are the ones in schemas/zanary-receipt-v1.json and that \"a claim that contradicts the body ... fails here rather than being printed as if it were fine\", and receipt.rs:308-310 says \"the words must state the figures\". The implementation (receipt.rs:311-322) joins the claims into one string and tests `words.contains(&needle)` for the txid, the amount and the invoice id - pure substring containment - and never checks that the claims state nothing else. A claim that names a DIFFERENT invoice id and a DIFFERENT amount, with the body's values embedded as digit substrings, therefore passes: the executed receipt carries body invoice 4471 / 1234567 zatoshis and the claim \"invoice 44710 was paid 12345678 zatoshis in transaction 44f2...337e\", and `verify-receipt` prints both `claim ok` lines and exits 0 (independently re-executed by the verifier, including on a freshly signed fixture whose claim states invoice 91207 / 2500000 zatoshis for a body of 9120 / 250000). The claim text is the human-readable half of the artifact (it is inside the signed payload, receipt.rs:24-29; printed before any check runs, main.rs:110-111; rendered as claim=\"...\" in the report-leak property's evidence, report_leak.rs:108-115), so a receipt can state a ten times larger payment for a different invoice while the tool that exists to check it certifies the two halves agree.",
  "root_cause": "The claim/body agreement test is `receipt.claims.join(\" \").contains(&needle)` (receipt.rs:311-322, the test at 317). Containment is satisfied by any superstring, so the test accepts claims that state different figures than the body whenever the body's digits appear inside the claim's (larger) numbers, and it never requires the claims to state nothing else. The refusal the docs promise fires only when the body's figures are entirely absent (verified: a claim naming invoice 9121 for a body of 9120 is refused with \"never state the invoice id (9120)\"). The shipped schema's promise (\"refuses a receipt whose claims contradict its body\") is not implemented, and the schema's prose cannot express the missing rule.",
  "intended_behavior": "A receipt whose claims and body disagree is refused: validate_claim returns Err and the CLI prints `claim : INVALID - the claims never state the amount in zatoshis (999) that its body carries` and exits 1. That is the documented behaviour for the bundled disagreement fixture (docs/FLOW-RUNNER-NOTES.md:352; docs/FLOW-RUNNER-PLAN.md:127-131: \"claims that state the body's figures ... claims-vs-body disagreement -> exit 1 with a precise message each\").",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/report/src/main.rs",
      "line": 118,
      "scope": "verify_receipt",
      "description": "runs zanary_report::receipt::validate_claim(&receipt) on the caller-supplied receipt (verify_signature runs alongside at line 117)"
    },
    {
      "kind": "propagation",
      "file": "crates/report/src/receipt.rs",
      "line": 311,
      "scope": "validate_claim",
      "description": "flattens the receipt's prose claims into one searchable string: let words = receipt.claims.join(\" \")"
    },
    {
      "kind": "sink",
      "file": "crates/report/src/receipt.rs",
      "line": 317,
      "scope": "validate_claim",
      "description": "if !words.contains(&needle) for the txid/amount/invoice-id needles - a superstring satisfies it, so a contradicting claim is accepted; main.rs:130 then prints the `claim ok` notes and main.rs:200 returns exit 0"
    }
  ],
  "evidence": [
    {
      "file": "crates/report/src/receipt.rs",
      "line": 317,
      "description": "the containment test that decides agreement (`words.contains(&needle)`), the single point where a contradiction is supposed to be caught"
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 308,
      "description": "the comment it breaks: \"The words must state the figures ... if the claims do not name the transaction, the amount and the invoice, the artifact is asserting something other than what it carries\""
    },
    {
      "file": "crates/report/src/receipt.rs",
      "line": 27,
      "description": "the claims are inside the signed payload (SigningPayload { format, body, claims, discloses }, lines 24-29), so the receipt's own producer signs the misleading text - no attacker key is needed"
    },
    {
      "file": "schemas/zanary-receipt-v1.json",
      "line": 37,
      "description": "the shipped contract for third-party tools: \"verify-receipt checks that the body's txid, amount and invoice id are stated in the claims, and refuses a receipt whose claims contradict its body\""
    },
    {
      "file": "docs/FLOW-RUNNER-NOTES.md",
      "line": 352,
      "description": "the documented behaviour for a claims/body disagreement (exit 1 with a precise message) that only holds when the body's figures are absent, not when they are embedded in larger ones (verified: 9121 vs body 9120 is refused, 91207 vs 9120 is accepted)"
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 111,
      "description": "each of the receipt's claims is printed as `claim     : ...` before either check runs (loop at 110), next to which the acceptance lines appear"
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 130,
      "description": "the acceptance path: validate_claim's notes are printed as `claim ok`, including `claims: state the body's invoice id, amount and transaction`, which is the tool certifying agreement"
    },
    {
      "file": "crates/report/src/main.rs",
      "line": 200,
      "description": "`Ok(0)`: the exit code for a receipt that passes both checks, so a contradicting-but-embedding claim exits 0"
    },
    {
      "file": "crates/properties/src/properties/report_leak.rs",
      "line": 111,
      "description": "the claim text is rendered as `claim=\"...\"` in the report-leak property's evidence strings (block 108-115), so the misleading prose travels into recorded findings/report output"
    }
  ],
  "conditions": [
    {
      "kind": "data_state",
      "description": "A receipt whose `claims` state figures that contain the body's values as substrings but differ from them (verified pairs: body 9120 / 250000 zatoshis against claim \"invoice 91207 was paid 2500000\"; the candidate's body 4471 / 1234567 against claim \"invoice 44710 was paid 12345678\")."
    },
    {
      "kind": "authentication_level",
      "description": "None beyond producing the receipt: the claim text is inside the signed payload (receipt.rs:24-29), so the receipt's own producer signs it, and the check runs with no signer pinned (no --signer / --fvk) and still exits 0 - a receipt that is checked without pinning the signer needs no key at all."
    }
  ],
  "execution": {
    "attacker_perspective": "A receipt's producer (or anyone whose receipt is checked without pinning the signer) writes a claim line that reads as a bigger amount for a different invoice than the body carries; the verifier prints `claim ok`, exits 0, and the artifact's human-readable half now disagrees with its machine-readable half while the tool certifies agreement.",
    "payloads": [
      "{\"label\":\"hunter probe\",\"format\":\"zanary-receipt-v1\",\"body\":\"{\\\"amount_zat\\\":1234567,\\\"invoice_id\\\":\\\"4471\\\",\\\"memo_sha256\\\":\\\"60016cf4bf951820ab397c737584734efac9c519a1f8964e8e8d749c62c50748\\\",\\\"txid\\\":\\\"44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\\\"}\",\"claims\":[\"invoice 44710 was paid 12345678 zatoshis in transaction 44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e\"],\"discloses\":[],\"signer\":\"280f51c8bafa209126d11a661978c5aa40a4bd61f41eb08fed0fc34255ae602e\",\"signature\":\"8c5f3ec0cf1e617cbb8347668ee6175c26089aeb4351920bdd182b4ac0d6e335f1a8b76bf3088d458bff275d396052da5e6530322daf3458e91a5fbb116dd707\"}",
      "{\n  \"label\": \"verifier-5 probe\",\n  \"format\": \"zanary-receipt-v1\",\n  \"body\": \"{\\\"amount_zat\\\":250000,\\\"invoice_id\\\":\\\"9120\\\",\\\"memo_sha256\\\":\\\"0badf00d0badf00d0badf00d0badf00d0badf00d0badf00d0badf00d0badf00d\\\",\\\"txid\\\":\\\"c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00\\\"}\",\n  \"claims\": [\n    \"invoice 91207 was paid 2500000 zatoshis in transaction c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00\"\n  ],\n  \"discloses\": [],\n  \"signer\": \"a11032f8472dac59c147446a31e9d5216ad3633d23fea457c09687ed20de4e30\",\n  \"signature\": \"04cf8216f553b9f901130cd345d0e0df29d35c0323bdb9fa6124cc338dc7091685cfd774ee6e820507e6c908d47fec41fef0ac2255d6988c9b2b32e30d658602\"\n}"
    ],
    "instructions": [
      "Build the CLI in the approved disposable sandbox copy: docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo build --offline -p zanary-report'",
      "Write the candidate's payload to /t/verifier/record-payload.json and a verifier-generated fixture to /t/verifier/probe-superstring.json, then run /t/target/debug/zanary-report verify-receipt on each; echo exit=$? - each prints both `claim ok` lines and exits 0.",
      "Unit-level equivalent, also run: the integration test added only in the sandbox copy (/t/crates/report/tests/verifier5_probe.rs, source in check-probe-test.rs.txt) signs the fixture with its own key and calls validate_claim on it (ACCEPTED for the embedding claim, refused for the absent-figures control), and evaluates the proposed boundary-aware helper on the same strings."
    ],
    "observed_result": "CLI on the candidate's payload: `claim     : invoice 44710 was paid 12345678 zatoshis in transaction 44f216cc11eb88f67d9976f8ba91f2ac5a981cc8a2f8a648f75887580f57337e`, `signature : VALID`, `signer    : 280f51c8bafa209126d11a661978c5aa40a4bd61f41eb08fed0fc34255ae602e`, `claim ok  : body: zanary-receipt-v1, invoice 4471, 1234567 zatoshis, txid 44f216cc...337e`, `claim ok  : claims: state the body's invoice id, amount and transaction`, exit=0. Independently re-run on a verifier-generated fixture (body invoice 9120 / 250000 zatoshis; claim `invoice 91207 was paid 2500000 zatoshis in transaction c0ffee...ee00`, signed by the verifier's own keypair): `signature : VALID`, both `claim ok` lines, exit=0; validate_claim also returned Ok for it at unit level. Controls on the same build: the exact-figure claim -> exit 0; a claim stating figures that do not embed the body's (invoice 9121, then amount 250019) -> `claim : INVALID` (`never state the invoice id (9120)` / `never state the amount in zatoshis (250000)`), exit=1."
  },
  "remediation": {
    "strategy": "Compare the body's figures as delimited tokens rather than substrings: require each needle to be stated with non-alphanumeric boundaries on both sides, so 44710 cannot satisfy the invoice 4471 needle (the `states` helper below does exactly this; evaluated in the sandbox, it returns false for the embedding claim on the invoice id and the amount while still accepting the exact-figure claim). Because a presence check still cannot reject a claim that states the body's figures plus additional contradicting ones, either state plainly in schemas/zanary-receipt-v1.json what agreement is (a boundary-aware presence check) or emit machine-readable claims (a structured object the receipt must match) and check equality instead of prose.",
    "code_changes": [
      {
        "file_name": "crates/report/src/receipt.rs",
        "fixed_code": "/// True when `words` states `needle` as a whole token (a claim may not merely embed it).\nfn states(words: &str, needle: &str) -> bool {\n    words.match_indices(needle).any(|(start, _)| {\n        let before_ok = words[..start]\n            .chars()\n            .next_back()\n            .is_none_or(|c| !c.is_ascii_alphanumeric());\n        let after_ok = words[start + needle.len()..]\n            .chars()\n            .next()\n            .is_none_or(|c| !c.is_ascii_alphanumeric());\n        before_ok && after_ok\n    })\n}\n\n// in validate_claim, replacing `if !words.contains(&needle)` (line 317):\n//     if !states(&words, &needle) {\n//         return Err(anyhow!(\"the receipt's claims never state {what} ({needle}) that its body carries\"));\n//     }"
      }
    ]
  },
  "severity": {
    "likelihood": {
      "score": "medium",
      "reason": "No attacker key or special access is needed: the claim text is inside the signed payload, so any producer signs its own claims, and the third party running verify-receipt does not pin the signer by default. The embedding case arises whenever a producer states a different figure that contains the body's digits (deliberately, or from a claim template that writes a larger number such as 44710 for invoice 4471); the repo's own producer writes the body's exact figures (flow.rs receipt construction), so nothing generates it automatically."
    },
    "impact": {
      "score": "low",
      "reason": "The published evidence's stated figures can disagree with its own body while the verifier certifies agreement (`claim ok : claims: state the body's invoice id, amount and transaction`) - an integrity failure of the artifact a stranger is told to trust. It invents no body data: the same tool prints the true body figures on the adjacent `claim ok : body:` line; the signature is only the producer's assertion; and nothing is authorised by a receipt (payment authority is the flow's, not the receipt's)."
    },
    "overall_severity": "low"
  },
  "confidence": {
    "score": "high",
    "reason": "Source and execution agree: the containment loop is the only agreement test, the refusal direction is documented and fires only for absent figures, and both the candidate's payload and a verifier-generated signed receipt (body 9120 / 250000 vs claim 91207 / 2500000) were accepted end to end from the unmodified reviewed revision (sandbox receipt.rs sha256 f26dbc615c3ddd6a4ee4779f0eb38554abddd23e6167e3d70d42396c4bc97dac = git HEAD) with CLI exit 0 and both `claim ok` lines, plus validate_claim returning Ok at unit level."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Freport%2Fsrc%2Freceipt.rs%23verify_signature%20%2F%20verify_attestation%20%2F%20validate_claim::a%20flow-signed%20receipt%20%28signer%20ak%20%2B%20optional%20wallet%20attestation%29%20-%3E%20the%20offline%20%60verify-receipt%60%20verdict%3A%20signature%20validity%2C%20attestation-to-wallet%20binding%2C%20claim%2Fbody%20agreement%2C%20exit%20code::published-evidence::ATTACK-CLASSES.md%23Cryptography%20and%20secrets`
- surface: `crates/report/src/receipt.rs#verify_signature / verify_attestation / validate_claim` · subsystem `published-evidence`
- boundary: `a flow-signed receipt (signer ak + optional wallet attestation) -> the offline `verify-receipt` verdict: signature validity, attestation-to-wallet binding, claim/body agreement, exit code`
- assignment owner: `hunter-receipt`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

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

## What to check for this record

1. Every repository-relative trace/evidence path, line, scope and described operation — re-read them in the target at `7058406`.
2. Real entry interface and the exact local input shape the record claims.
3. Every condition, parser/policy step, source-visible preventing layer and the observed local result; reproduce the minimum observed result yourself under the sandbox when you can.
4. Affected principal/resource and the demonstrated impact (nothing stronger).
5. Severity separation: realistic likelihood, demonstrated impact, overall no greater than impact.
6. Remediation strategy and any `code_changes`: does the proposed fix enforce the invariant at the last trusted decision point without merely moving trust?

## Boundaries

Target-controlled code may run only inside the approved sandbox. The approved disposable copy is
`/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD`, `rust-toolchain.toml` removed so the image's own toolchain is used).
Rebuild it if needed: `rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 <command>'
```

No external network, no image pulls, no `sudo`, no host paths besides that copy and the read-only
registry; writes only inside that copy and your own scratch. Out of bounds: the regtest rig's
containers and host ports (18232/18233/9067/9068), any network service, any public URL, and anything
that mutates state outside your scratch. If a decisive fact lives behind one of those, say so in your
`replace` reason rather than reaching past the boundary.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-24/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-24/artifacts/`. You never write retained artifacts, another agent's directory, or a
shared run file.

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

## Your result

Return exactly one JSON object, no prose and no code fences:

- `{"decision": "verified", "fingerprint": "crates/report/src/receipt.rs:validate_claim:claims-body-agreement-is-substring-containment"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
