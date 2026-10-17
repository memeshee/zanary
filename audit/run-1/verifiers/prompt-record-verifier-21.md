# Phase-5 record verifier — verifier-21 on `crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction",
  "title": "The bounded payer writes every payment record from literals (Orchard -> Orchard, Shielded, FullPrivacy) and never applies the recipient-policy guard, so a payment whose transaction crosses a shielded pool boundary (a Sapling receiver) would be recorded and scored as a same-pool, fully-shielded, FullPrivacy payment",
  "description": "payer::run pays each invoice it admitted through ops::propose_shielded_payment / ops::build_sign_broadcast and then records the payment at payer.rs:339-362, where pool_in: Pool::Orchard (349), pool_out: Pool::Orchard (350), kind: PaymentKind::Shielded (351) and policy: Some(\"FullPrivacy\") (352) are constants: none is derived from the proposal or the transaction, and payer.rs never reads the proposal back (0 occurrences of proposal.steps / payment_pools / any transaction decryption, against ops.rs:1076-1126 where `pay` decrypts the mined transaction). observation/src/lib.rs:28-30 defines PaymentKind::Shielded as \"shielded source and shielded destination, same pool\" and 56-60 defines pool_out as the payment's destination pool and `policy` as \"the privacy policy argument the flow passed to its wallet layer, verbatim\" - the payer passes no such argument. This path also never calls ops::required_policy_for_recipient + ops::check_policy (payer.rs: 0 call sites; every other payment path does: ops.rs:563-566, 777, 1019-1020; flow.rs:633-634, 826-827), and ops::propose_shielded_payments (payer.rs:306-307 -> ops.rs:900-961) applies SpendPolicy::shielded_pools([Orchard]) to input sources only (zcash_client_backend 0.24.0 input_selection.rs:476-489, 966-968, 999-1003). The recipient comes from the merchant's invoice file, deserialized directly (payer.rs:495-519, no ZIP-321 cross-check unlike flow.rs:1740-1768), and decide only string-compares it against the grant allowlist (payer.rs:100); an empty or absent allowlist admits every receiver (printed at payer.rs:222-224). A merchant-supplied invoice naming a Sapling receiver therefore reaches the wallet layer: verifier-2 reproduced this in the offline sandbox - the payer's own intake took the dev account's bare zregtestsapling1 receiver and a Sapling-only UA to \"Must scan blocks first\" (fundless stub wallet, identical to the Orchard control), while required_policy_for_recipient + check_policy refuse those same recipients under FullPrivacy with \"would reveal the amount (Orchard -> Sapling pool crossing)\" (ops.rs:243-246). The record then decides every verdict derived from it: policy-downgrade (Severity::Disqualifying) computes its explicit signal from Payment.policy (76-89; \"FullPrivacy\" matches no downgrade fragment) and its implicit signal from pool_in (shielded) plus pool_out in {Transparent, CrossPool} (92-101), so the payer's constant record makes it PASS - reproduced with zanary_properties::evaluate() in the sandbox, where the fixtures' honest cross-pool record (fixtures.rs:243-246) FAILs as a positive control. Nothing re-derives a pool leg afterwards: report/src/verify.rs re-checks tx height / receiver / memo / pool totals as report evidence and the report crate never mentions pool_in/pool_out, and no property compares run.payments against the ledger or the chain. Calibration: the flow runner writes the same Orchard/Orchard/Shielded constants (flow.rs:687-690, 984-987) but records the real policy token and does gate recipients, and the property's implicit signal does not cover pool_out Sapling at all - the record lie is what turns an undetected crossing into a recorded, scored \"fully private\" payment.",
  "claimed_root_cause": "Two coupled defects on the payer's pay path: (1) the record fields the leak properties read are literals - pool_in/pool_out/kind/policy at payer.rs:349-352 are never derived from the proposal's payment pools or from the built transaction, so they describe what the payer assumed rather than what it built; (2) the payer is the only payment path in the workspace that does not call ops::required_policy_for_recipient + ops::check_policy before proposing (0 call sites in payer.rs against ops.rs:563-566/777/1019-1020 and flow.rs:633-634/826-827), so a merchant-supplied recipient that makes the payment reveal more than FullPrivacy (a Sapling receiver, which the target's own intake accepts and hands to the wallet layer) is paid instead of refused, and is then recorded as Orchard -> Orchard Shielded FullPrivacy.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "crates/probe/src/payer.rs",
      "line": 298,
      "scope": "payer::run - the payment loop over the invoices decide allowed",
      "description": "The merchant-supplied receiver (deserialized straight from the invoice JSON at payer.rs:495-519, with no ZIP-321 cross-check unlike flow.rs:1740-1768) is decoded into the address the payment will be made to; there is no recipient-kind or privacy-policy gate anywhere on this path (payer.rs has 0 occurrences of check_policy / required_policy_for_recipient)."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/ops.rs",
      "line": 900,
      "scope": "ops::propose_shielded_payments",
      "description": "The recipient becomes a ZIP-321 payment handed to propose_transfer with SpendPolicy::shielded_pools([Orchard]) (inputs only; 929) and an Orchard change strategy (931-936), so nothing constrains the recipient's own pool. Reproduced: a bare Sapling receiver and a Sapling-only UA pass this intake and reach the wallet layer (\"Must scan blocks first\" in a fundless stub wallet), while required_policy_for_recipient + check_policy refuse the same recipients under FullPrivacy (ops.rs:243-246)."
    },
    {
      "kind": "propagation",
      "file": "crates/probe/src/ops.rs",
      "line": 360,
      "scope": "ops::build_sign_broadcast",
      "description": "Whatever the proposal built is proved, signed, read back from the wallet (396-402) and sent to the indexer (407-415) without any comparison against the invoice the payer decided about and without inspecting the transaction's pools."
    },
    {
      "kind": "sink",
      "file": "crates/probe/src/payer.rs",
      "line": 349,
      "scope": "payer::run - the Payment record written for the payment just broadcast",
      "description": "pool_in, pool_out, kind and policy are written as constants (349-352: Orchard / Orchard / Shielded / FullPrivacy), so the run's evidence - and the findings, report and index built from it - assert a same-pool, fully-shielded, FullPrivacy payment regardless of the transaction's real shape; policy-downgrade (Disqualifying) reads exactly these fields and passes (reproduced)."
    }
  ],
  "evidence": [
    {
      "file": "crates/probe/src/payer.rs",
      "line": 349,
      "description": "the four fields policy-downgrade reads are literals: pool_in: Pool::Orchard (349), pool_out: Pool::Orchard (350), kind: PaymentKind::Shielded (351), policy: Some(\"FullPrivacy\") (352); payer.rs reads nothing back (0 occurrences of proposal.steps / payment_pools / decrypt - verifier-2's sandbox count over the compiled file)"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 243,
      "description": "required_policy_for_recipient maps a Sapling receiver (or a UA with a Sapling receiver and no Orchard receiver, 250-254) to AllowRevealedAmounts because the payment would reveal \"the amount (Orchard -> Sapling pool crossing)\" (243-246)"
    },
    {
      "file": "crates/probe/src/ops.rs",
      "line": 1020,
      "description": "the CLI pay path calls check_policy before proposing (1019-1020); ops.rs:563-566 (anchor), ops.rs:777 (shield) and flow.rs:633-634 / 826-827 do the same; payer.rs has no equivalent (0 occurrences, verifier-2 sandbox count)"
    },
    {
      "file": "crates/properties/src/properties/policy_downgrade.rs",
      "line": 92,
      "description": "the implicit-downgrade signal is computed from Payment.pool_in/pool_out alone (92-101; reveal requires pool_out Transparent|CrossPool) and the explicit one from Payment.policy (76-89; \"FullPrivacy\" matches no fragment), so the constant record decides the verdict - reproduced: zanary_properties::evaluate() returns PASS for the payer's record and FAIL for the fixtures' honest cross-pool control"
    },
    {
      "file": "crates/properties/src/properties/policy_downgrade.rs",
      "line": 55,
      "description": "policy-downgrade is Severity::Disqualifying (54-56), so the false PASS is published as a clean verdict for a run whose payment actually crossed a shielded pool boundary"
    },
    {
      "file": "crates/observation/src/lib.rs",
      "line": 28,
      "description": "PaymentKind::Shielded is defined as \"shielded source and shielded destination, same pool\" (28-30); pool_out is the payment's destination pool and `policy` the \"privacy policy argument the flow passed to its wallet layer, verbatim\" (56-60) - recording Orchard/Shielded/FullPrivacy for a Sapling-output payment is false by the model's own definitions, and the payer passed no such argument"
    },
    {
      "file": "crates/properties/src/properties/fixtures.rs",
      "line": 243,
      "description": "the fixture that models a cross-pool payment records the real destination pool (pool_in Orchard, pool_out Transparent, kind CrossPool, policy AllowRevealedAmounts; 243-246) and drives the property's failing path - used here as the positive control that the property bites when the record is honest"
    },
    {
      "file": "crates/probe/src/core.rs",
      "line": 84,
      "description": "the harness chain activates Sapling at height 1 (84-96) while NU6.1/6.2/6.3 (Ironwood) stay unactivated, so a Sapling output is the shielded cross-pool delivery this build can produce"
    },
    {
      "file": ".cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zcash_client_backend-0.24.0/src/data_api/wallet/input_selection.rs",
      "line": 966,
      "description": "dependency source (version pinned by Cargo.toml:20, vendored read-only): GreedyInputSelector::propose_transaction delivers an Address::Sapling recipient through a Sapling output with payment_pools[idx] = SAPLING (966-968; a Sapling-only UA at 999-1003), and SpendPolicy::shielded_pools constrains input sources only (476-489) - so the transaction built for a Sapling-receiver invoice carries the Sapling leg the record denies (source-corroborated; not executed - see blockers)"
    },
    {
      "file": "crates/probe/src/flow.rs",
      "line": 687,
      "description": "calibration: the flow runner writes the same Orchard/Orchard/Shielded constants (687-690, 984-987) but records the real policy token (Some(policy.name())) and calls the guard at 633-634 / 826-827 - the template pattern is shared, while the missing guard is specific to the payer"
    },
    {
      "file": "crates/report/src/verify.rs",
      "line": 1,
      "description": "`zanary-report verify` re-derives tx height / receiver / memo / pool-total evidence from the chain, and the report crate never mentions pool_in/pool_out (0 occurrences): nothing re-derives or cross-checks the recorded pool legs"
    },
    {
      "file": "crates/probe/src/payer.rs",
      "line": 100,
      "description": "the decision only string-compares the receiver against the grant allowlist; an empty or absent allowlist admits every receiver and is printed as such at 222-224"
    }
  ],
  "blockers": [
    "The decisive step - that the proposal/transaction built for a Sapling-receiver invoice carries a Sapling output - could not be executed in the approved sandbox: it needs a wallet holding a spendable Orchard note, and no offline route exists (zcash_client_backend's test-wallet framework needs the test-dependencies feature set -> proptest/ambassador/assert_matches, none of which are in the read-only registry cache - verifier-2 confirmed the cache holds only incrementalmerkletree/rand_chacha/tempfile of that set - and --network=none permits no fetch). The leg is Sapling by source (input_selection.rs:966-968, 999-1003); execution is what is missing.",
    "The end of the path - the broadcast, the mined transaction's shape, and the report/index published from the run - needs the regtest rig (its containers and host ports 18232/18233/9067/9068), which is out of bounds.",
    "Whether a Sapling-receiver invoice ever reaches a payer in practice is a deployment fact: every recorded run on this host and the CI demo issue Orchard-only invoices with a 3-receiver allowlist (data/grants/*.json, data/demo-payer/invoices/*.json, scripts/bounded-payer-demo.sh), while the payer's contract is any invoice directory and no observation of a Sapling-receiver invoice reaching a payer exists."
  ],
  "validation_plan": {
    "local": "In a sandbox whose registry cache is populated (or with network for the one fetch), build zcash_client_backend and zcash_client_sqlite with `test-dependencies`, fund one Orchard note in their data_api::testing test wallet, then call propose_transfer with the payer's exact arguments (params = core::regtest_network(), account, GreedyInputSelector::new(), SingleOutputChangeStrategy::new(Zip317, None, ShieldedPool::Orchard, DustOutputPolicy::default()), ConfirmationsPolicy::default(), SpendPolicy::shielded_pools([Orchard]), request = one zip321 payment to a zregtestsapling1... receiver carrying the invoice memo - ops.rs:929-961) and print proposal.steps()[0].payment_pools() - expected {0: PoolType::SAPLING}: the payment leg is Sapling while payer.rs:350 would record Orchard. Without that framework, verifier-2 already re-established everything short of the proposal in /t (crates/probe/src/bin/verifier2_check.rs; output check-verify2.txt): payer.rs:349-352 are literals and payer.rs calls neither guard; the Sapling receiver reaches the wallet layer through the payer's own intake while required_policy_for_recipient + check_policy refuse it; zanary_properties::evaluate() gives the constant record policy-downgrade PASS (and the fixtures' honest cross-pool record FAIL).",
    "deployment": "Owner-observed on the regtest rig (throwaway funds, no external network, nothing touched outside a fresh --out directory): write one invoice JSON whose receiver is the dev account's own Sapling receiver with a self-consistent URI (zcash:?address=<zregtestsapling1...>&amount=<...>) and a short memo, plus a grant with an empty allowlist (or that receiver listed); run `zanary-probe payer --grant <grant.json> --invoices <dir> --out <fresh-dir>`; then read (a) run.json payments[0].pool_in/pool_out/kind/policy (expected the constants Orchard/Orchard/Shielded/FullPrivacy), (b) findings.json / report.json policy-downgrade verdict (expected PASS), (c) the txid via getrawtransaction/decoderawtransaction (expected a Sapling output and a non-zero sapling valueBalance). Non-destructive control on the same invoice (refuses before proposing or broadcasting): `zanary-probe pay --invoice <the same file> --policy FullPrivacy` must refuse at ops.rs:1019-1020 with \"would reveal the amount (Orchard -> Sapling pool crossing)\"."
  }
}
```

## Linked ledger unit

- coverage_id: `crates%2Fprobe%2Fsrc%2Fops.rs%23propose_shielded_payment%20%2F%20build_sign_broadcast::the%20authority%20check%20%28decide%29%20-%3E%20the%20transaction%20actually%20built%20and%20broadcast::bounded-payer::ATTACK-CLASSES.md%23Business%20logic`
- surface: `crates/probe/src/ops.rs#propose_shielded_payment / build_sign_broadcast` · subsystem `bounded-payer`
- boundary: `the authority check (decide) -> the transaction actually built and broadcast`
- assignment owner: `hunter-broadcast`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/ops.rs",
      "crates/observation/src/lib.rs"
    ],
    "invariant": "the pool legs and privacy policy a payer payment records describe the transaction that was built, not a constant template",
    "method": "source",
    "result": "payer.rs:347-352 writes pool_in: Orchard, pool_out: Orchard, kind: Shielded, policy: Some(\"FullPrivacy\") as literals for every payment, and payer.rs never reads anything back from the proposal it built (no proposal.steps, no payment_pools, no transaction decryption anywhere in the file, unlike ops.rs:1076-1126 where `pay` decrypts the mined transaction). observation/src/lib.rs:28-36 and 56-60 define those fields as the payment's real pools and the 'privacy policy argument the flow passed to its wallet layer, verbatim' \u2014 the payer passes no such argument.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/ops.rs",
      "crates/probe/src/payer.rs"
    ],
    "invariant": "a merchant-supplied recipient whose payment would reveal more than FullPrivacy is refused before anything is proposed or built, as every other payment path in the workspace refuses it",
    "method": "local",
    "result": "Observed in the approved sandbox (check-intake.txt, 2 tests passed, exit 0): an invoice-style payment to a Sapling receiver (bare zregtestsapling1..., and a unified address with a Sapling receiver and no Orchard receiver) is ACCEPTED by the target's own intake in ops::propose_shielded_payments (it reaches the wallet layer: 'Must scan blocks first', with the invoice memo and with an empty memo), while ops::required_policy_for_recipient + ops::check_policy refuse the same recipient under FullPrivacy with 'would reveal the amount (Orchard -> Sapling pool crossing)'. payer.rs contains 0 occurrences of either function, against ops.rs:566 / ops.rs:777 / ops.rs:1020 / flow.rs:634 / flow.rs:827 on the other payment paths. A bare transparent receiver is refused inside the target's intake by zip321's memo rule, so the transparent variant is not reachable.",
    "artifact": "agents/hunter-broadcast/artifacts/check-intake.txt"
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs"
    ],
    "invariant": "the run's per-invoice decisions and artifacts are committed even when a payment step fails after earlier payments were broadcast",
    "method": "source",
    "result": "payer.rs:246-270 decides every invoice first and only prints; the payment loop (291-363) propagates each step with `?` (decode 298-303, memo 304-305, propose 306-307, build_sign_broadcast 308, mine_and_resync 309, mined_height_of 310); ledger.jsonl and the run/report artifacts are written afterwards (408-411) and --expect-refused is evaluated later still (469-478). A mid-loop failure therefore returns Err with nothing recorded although earlier iterations already broadcast, and no code path in payer.rs can produce the `error:` refusal reason that properties/src/properties/payer_policy.rs:66-74 exists to flag.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/payer.rs",
      "crates/probe/src/flow.rs"
    ],
    "invariant": "every invoice the decision layer admits can be executed by the payment loop (the decision covers the whole invoice, not only budget/cap/allowlist fields)",
    "method": "local",
    "result": "Observed in the approved sandbox (check-intake.txt): with the documented grant shape (empty allowlist, cap 10,000,000 zat, budget 50,000,000 zat) an invoice whose receiver is 'not-an-address' and whose amount is 5,000,000 is decided allowed=true ('paid: within the cap ... and the budget'), and Address::decode then returns NONE at payer.rs:298; an invoice whose memo is 600 bytes is likewise decided allowed=true and rejected by MemoBytes::from_bytes at payer.rs:304. The same run shows the payer's intake (serde, payer.rs:515-516) accepting an invoice whose URI pays the Orchard UA but whose receiver says Sapling, which flow.rs:1740-1768 (ops::pay --invoice) refuses as 'an invoice that disagrees with itself'.",
    "artifact": "agents/hunter-broadcast/artifacts/check-intake.txt"
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/properties/src/properties/policy_downgrade.rs",
      "crates/properties/src/properties/fixtures.rs",
      "crates/properties/src/properties/payer_policy.rs"
    ],
    "invariant": "the leak verdict for a payment's pools is derived from the chain/transaction rather than from the writer's self-report, and the fixture set pins the encoding of a cross-pool payment",
    "method": "source",
    "result": "policy_downgrade.rs:76-101 computes both signals from Payment.policy and Payment.pool_in/pool_out alone (an explicit token match and pool_out in {Transparent, CrossPool}); nothing in the properties crate re-derives a pool leg from a transaction, and payer_policy.rs:63-103 likewise checks only the ledger's own amount/receiver/revoked fields against the grant. fixtures.rs:240-246 models a cross-pool payment with the real destination pool (pool_out Transparent, kind CrossPool, policy AllowRevealedAmounts), i.e. the record is the only evidence there is.",
    "artifact": null
  },
  {
    "agent_id": "hunter-broadcast",
    "reviewed_paths": [
      "crates/probe/src/core.rs"
    ],
    "invariant": "the chain the payment is built for activates the pools that decide what the transaction can reveal",
    "method": "source",
    "result": "core.rs:81-91 pins LocalNetwork with sapling: Some(1) (active from height 1) and nu6_1/nu6_2/nu6_3 unactivated, so an Orchard-receiver payment is an Orchard output and a Sapling receiver is the shielded cross-pool delivery this build can actually produce; the harness node (zcashd v6.20.0) matches.",
    "artifact": null
  }
]
```

## What to check for this record

1. The source path is real and supports only the `claimed_root_cause` stated.
2. Every listed blocker is decisive and not already answerable locally.
3. The candidate names a boundary and a possible concrete result rather than a generic concern.
4. At least one `validation_plan` field is present and exact: `local` uses a bounded fixture; `deployment` asks the owner to observe a configuration, identity, route, policy or runtime fact. Do not invent a plan for an inapplicable context, and never send audit traffic to a deployment.
5. The fingerprint matches prior/current records for the same root cause.

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

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-21/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-21/artifacts/`. You never write retained artifacts, another agent's directory, or a
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

- `{"decision": "verified", "fingerprint": "crates/probe/src/payer.rs:payment-record-pool-legs-and-policy-are-constants-not-derived-from-the-built-transaction"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
