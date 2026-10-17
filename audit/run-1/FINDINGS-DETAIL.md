# Findings detail — zanary run 1

Confirmed `medium`, `high` and `critical` records, with the full source path, the target-native bounded reproduction and the fix. Reproductions are target-neutral: they use the target's own CLI, tests or library calls inside an OS-enforced sandbox (`--network=none`, read-only registry, scratch only), and stop at the minimum observed effect.

## Composite action expands caller-supplied with: inputs into shell script text

`zanary/composite-action-input-shell-expansion` · overall **medium** (likelihood low, impact high) · confidence high

action.yml is published as a composite action that third-party workflows run (`uses: memeshee/zanary@main` is the documented whole setup: action.yml:7, docs/CI.md:49). Its five inputs (flow, expect, anchor, ttl-days, out; action.yml:23-46) are free-form strings with no validation, and the action places their `${{ inputs.* }}` values in the script text of its `run:` steps -- the gate step (:95, :100, :103, :106, :116, :117), the always-run job summary step (:140) and the verdict step (:177-178) -- instead of receiving them through env: or argv. The runner substitutes those expressions into the script before bash parses it, so a value carrying shell syntax (a quote plus ;, $( ), backticks, #) is executed by the calling job's shell. The gate step also writes the value derived from inputs.out into GITHUB_OUTPUT as one raw, unescaped line (:107), so the value leaves the step as a step output and is re-interpolated into script text at :136 (job summary, `if: always()`) and :153 (rig diagnostics); the verdict step interpolates the step's exit_code output the same way (:175, numeric in this action's flow). Nothing in this repository triggers it: the documented consumer setup passes literal values (docs/CI.md:49-55, the flow literal at :51) and this repository's own gate passes static matrix entries defined in the same file (.github/workflows/privacy-gate.yml:87-93 from the matrix at :71-80). The defect is in the action as a library: it hands every consumer that templates one of the five inputs into `with:` a code-execution sink in that consumer's own job.

**Trace**

1. *entrypoint* — `action.yml:23` (composite action inputs): The action accepts five caller-supplied strings (flow, expect, anchor, ttl-days, out; lines 23-46) with no format validation; a third-party workflow supplies them in its own repository via with:.
1. *propagation* — `action.yml:95` (step run the gate - run: script text): The runner substitutes ${{ inputs.flow }} into the script text (a double-quoted assignment) before bash parses it; the identical pattern sits at lines 100, 103, 106, 116, 117 and in the job summary / verdict steps (140, 177-178). Reproduced locally: with a payload in flow the injection executed here, and also at :100 when the value was invalid.
1. *propagation* — `action.yml:107` (step run the gate - step output write): The out value is appended raw to GITHUB_OUTPUT (no key/newline escaping), so it also leaves the step as a step output and re-enters later scripts at 136, 153 and 175. Reproduced locally: the recorded value kept its $( ) text verbatim and executed again when the job summary step was rendered with it.
1. *sink* — `action.yml:103` (step run the gate - command context): OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }}, so the expansion lands in a double-quoted assignment in command position: a value with $( ), backticks, a closing quote plus ;, or # is executed by the caller shell (independently reproduced on this line and on line 95 with marker payloads in the offline sandbox).

**Evidence**

- `action.yml:95` — FLOW is assigned from ${{ inputs.flow }} inside the run the gate step - flow is expanded into script text.
- `action.yml:100` — second expansion of flow in the same step (the no-flow-definition error path) - a payload runs here as well (reproduced: two executions for one invalid value).
- `action.yml:103` — OUT is assigned from ${{ github.workspace }} and ${{ inputs.out }} - out expanded into a command context (the verified local run B).
- `action.yml:106` — the anchor input is expanded into script text in the [ anchor = true ] test.
- `action.yml:116` — expect is expanded inside the probe command line (--expect argument).
- `action.yml:117` — ttl-days is expanded the same way (--ttl-days argument).
- `action.yml:107` — the echo of out=$OUT into $GITHUB_OUTPUT - raw write, no newline escaping; the value leaves the step as a step output and a newline in it can forge further output lines (reproduced: an injected payload=owned key).
- `action.yml:136` — OUT is assigned from ${{ steps.gate.outputs.out }} in the always-run job summary step - the stored value re-enters script text (same at 153 in rig diagnostics); verified by execution.
- `action.yml:175` — code is assigned from ${{ steps.gate.outputs.exit_code }} in the verdict step - the output that decides the build result is also script text (numeric in this action's flow).
- `action.yml:140` — the title argument of the job summary step expands inputs.flow and inputs.expect; that step runs with if: always().
- `action.yml:177` — inputs.expect is expanded inside the verdict step case branches (also line 178).
- `docs/CI.md:51` — the documented consumer setup passes literal values (flow: flows/my-checkout.json), which is why no in-repo caller triggers the injection.

**Dummy principal and affected resource:** Whoever controls a value that a consumer workflow passes into the action inputs -- a contributor-influenced branch/PR-derived string, a reusable-workflow parameter, a dispatch input, or the consumer's own templated value -- gets the calling job's shell: the payload runs in the action's steps with that job environment (GITHUB_* variables, the step outputs, any job/step environment the step can see). A consumer that passes a literal cannot be attacked; the defect is that the action makes the templating pattern dangerous.

**Native input / invocation**

```
$(printf x >> /t/verifier10/count-flow.txt; printf 'flows/stub.json')
```
```
$(printf x >> /t/verifier10/count-out.txt; printf 'x$(printf y >> /t/verifier10/count-roundtrip.txt)')
```
```
$(printf 'x\npayload=owned')
```
- Read action.yml from the read-only target (commit 7058406, worktree clean) and assert the verbatim text of every interpolation site: :95, :96, :100, :103, :106, :107, :116, :117, :136, :140, :153, :175, :177, :178.
- Render the gate step's run-block (verbatim lines 92-129) with the literal expression substitution the Actions runner documents (plus ${{ github.workspace }} and ${{ github.action_path }}), one payload value per run: flow kept valid, flow made invalid (so :100 expands it again), out, and out with a newline; all other inputs benign literals.
- Run each rendered script with `bash --noprofile --norc -eo pipefail` inside `docker run --rm --network=none --cpus=6 --memory=3500m --pids-limit=1024 --user 1000:1000 -v <disposable copy>:/t -w /t/verifier10/action rust:1-slim`, with GITHUB_OUTPUT / GITHUB_WORKSPACE / GITHUB_ACTION_PATH / GITHUB_STEP_SUMMARY set and a placeholder ./target/release/zanary-probe (the real rig is out of bounds); each payload only appends one byte to a counter file so the execution count is visible.
- Clear the counters and the output file first, and run a control pass with every input a literal.
- Then render the always-run job summary step (action.yml:136-147) with the exact out= value the gate step recorded in GITHUB_OUTPUT and execute it the same way (the step-output round trip), with a benign-value render as the control; stop at the first observed boundary result.

**Observed output:** Independently reproduced in the offline sandbox with command-substitution payloads (no quote-breaking needed) against the verbatim gate run-block, rendered with literal expression substitution as the runner documents it and with a placeholder probe. Control (all inputs literal): the step ran normally, wrote out=/t/verifier10/ws/z and exit_code=0, and no counter file appeared. Flow payload (valid value): executed at :95 (counter x) while the step still finished normally, exit 0, with its own outputs. Flow payload (invalid value): executed twice (counter xx), at :95 and in the :100 error echo, then exited 2. Out payload: executed at :103 (counter x) and the step exited 0 having written the raw value out=/t/verifier10/ws/x$(printf y >> /t/verifier10/count-roundtrip.txt) and exit_code=0 to GITHUB_OUTPUT. Rendering the always-run job summary step with that recorded value executed the embedded fragment at :136 (counter y), while the benign-value control executed nothing. A newline in the out value made GITHUB_OUTPUT contain out=/t/verifier10/ws/x, payload=owned and exit_code=0, i.e. an injected output key. Nothing outside the disposable copy was written; --network=none, placeholder probe, no rig and no live runner. The only element not executed here is the runner's own substitution step (documented behavior; a live runner is out of bounds).

**Conditions and containment:** A calling workflow passes a non-constant value (a reusable-workflow parameter, a dispatch or matrix value, or an event-derived string) to one of flow, expect, anchor, ttl-days, out. No caller in this repository does that: the documented setup passes literals (docs/CI.md:49-55) and .github/workflows/privacy-gate.yml:87-93 passes static matrix entries defined in the same file at :71-80, so nothing fires here.

**Remediation:** Keep inputs as data on both sides of every step: declare them under env: and reference the environment variables inside the scripts; validate the shapes the action actually needs (flow/out caller-workspace-relative paths; expect in {pass, fail}; anchor in {true, false}; ttl-days a positive integer) and fail early with a clear error; keep the existing three-candidate flow search operating on the env value; write out to GITHUB_OUTPUT with the multiline-safe key<<DELIM form; and in the job summary, rig diagnostics and verdict steps pass steps.gate.outputs.out / steps.gate.outputs.exit_code through env: instead of re-interpolating them.

```rust
// action.yml
    - name: run the gate
      id: gate
      shell: bash
      working-directory: ${{ github.action_path }}
      env:                                    # every input arrives as data
        INPUT_FLOW: ${{ inputs.flow }}
        INPUT_OUT: ${{ inputs.out }}
        INPUT_EXPECT: ${{ inputs.expect }}
        INPUT_ANCHOR: ${{ inputs.anchor }}
        INPUT_TTL_DAYS: ${{ inputs.ttl-days }}
      run: |
        set -o pipefail
        case "$INPUT_EXPECT" in pass|fail) ;; *) echo "::error::expect must be pass or fail"; exit 2 ;; esac
        case "$INPUT_TTL_DAYS" in ''|*[!0-9]*) echo "::error::ttl-days must be a positive integer"; exit 2 ;; esac
        FLOW="$INPUT_FLOW"
        for candidate in "$GITHUB_WORKSPACE/$FLOW" "$GITHUB_ACTION_PATH/$FLOW" "$FLOW"; do
          if [ -f "$candidate" ]; then FLOW="$candidate"; break; fi
        done
        if [ ! -f "$FLOW" ]; then echo "::error::ZANARY gate: no flow definition at $INPUT_FLOW"; exit 2; fi
        OUT="$GITHUB_WORKSPACE/$INPUT_OUT"; mkdir -p "$OUT"
        anchor_flag=""
        if [ "$INPUT_ANCHOR" = "true" ]; then anchor_flag="--anchor"; fi
        { echo "out<<ZANARY_OUT"; printf '%s\n' "$OUT"; echo "ZANARY_OUT"; } >> "$GITHUB_OUTPUT"
        set +e
        timeout "${ZANARY_GATE_TIMEOUT_SECONDS:-2700}" ./target/release/zanary-probe test \
          --definition "$FLOW" --out "$OUT" --expect "$INPUT_EXPECT" \
          --ttl-days "$INPUT_TTL_DAYS" $anchor_flag --summary "$OUT/test.json" 2>&1 | tee "$OUT/gate.log"
        code=${PIPESTATUS[0]}
        if [ "$code" = "124" ]; then
          echo "::warning::the gate did not finish within ${ZANARY_GATE_TIMEOUT_SECONDS:-2700}s - treating it as a rig failure"
          code=2
        fi
        echo "exit_code=$code" >> "$GITHUB_OUTPUT"
        exit 0
    # Later steps (job summary, rig diagnostics, verdict) take GATE_OUT / GATE_EXIT
    # through env: (env: GATE_OUT: ${{ steps.gate.outputs.out }}) and never
    # re-interpolate ${{ }} inside script text.

```

## The grant's budget bounds invoice principal, not the cost of paying: per-payment fees and the invoice count sit outside the authority

`zanary/payer-budget-excludes-fees` · overall **medium** (likelihood high, impact medium) · confidence medium

docs/BOUNDED-PAYER.md defines budget as "the most the run may spend in total" (line 28), and observation::PayerRun::spent_zat() labels its sum "Zatoshis actually spent under this grant" (observation/src/lib.rs:233-234). decide()'s budget rule (payer.rs:119-128) adds only invoice amounts, and the payer-policy property re-derives exactly the same number (payer_policy.rs:77-90), while the payer's own funding model budgets one ZIP-317 fee per payment at 10_000 zat (payer.rs:528-530; the runner's fee budget agrees, flow.rs:1165-1177) — a cost the grant never counts. With no minimum invoice amount (Invoice.amount_zat is a bare u64, flow.rs:1544) and no cap on how many invoices a run may load, decide and pay (load_invoices, payer.rs:495-520), a merchant can make the run's real spend dwarf the granted budget: 1-zat invoices fill a 1,000-zat budget with 1,000 payments and ~10,000,000 zat of fees, each payment also costing 11 mined blocks and a wallet sync (payer.rs:309, ops.rs:59). The ledger and payer-policy report 1,000 of 1,000 zat spent. Re-verified by execution: the real decide() admits the full thousand and the real payer-policy property passes the resulting ledger. Calibration: the flow runner funds fees the same way but holds no grant, so only the payer's authority claim is contradicted.

**Trace**

1. *entrypoint* — `crates/probe/src/payer.rs:495` (load_invoices): the merchant-supplied invoice directory is read whole: every *.json in it is loaded (sorted by name) with no count cap, no size cap and no minimum amount per invoice (payer.rs:495-520)
1. *propagation* — `crates/probe/src/payer.rs:119` (decide() budget rule): only invoice principal is accumulated (spent_zat.saturating_add(invoice.amount_zat), 119; refuse only when after > budget_zat, 120): a 1-zat invoice is always affordable until the nominal budget is exhausted
1. *propagation* — `crates/probe/src/payer.rs:278` (funding): shortfall(&payer, &to_pay) (523-533) budgets sum(amounts) + 10_000 zat per payment (530) and flow::top_up_for_target (286; flow.rs:1202) mines/shields that headroom from outside the grant — the fee spend is real to the operator but invisible to the authority
1. *sink* — `crates/probe/src/payer.rs:291` (payment loop): one shielded transaction (307-308) plus mine_and_resync(CONFIRM_BLOCKS = 11) (309; ops.rs:59) per allowed invoice: the run's real cost multiplies with the invoice count while the ledger reports only principal

**Evidence**

- `crates/probe/src/payer.rs:530` — the payer's own fee model: `needed = needed.saturating_add(10_000 * to_pay.len() as u64)` — comment: one ZIP-317 fee per payment at 2 actions; the runner's fee_budget_zat models per-invoice settlement the same way (flow.rs:1165-1177, 10_000 zatoshis each)
- `crates/probe/src/payer.rs:119` — the budget rule compares only spent+amount (120) against budget_zat — fees never enter the comparison
- `crates/properties/src/properties/payer_policy.rs:77` — payer-policy re-derives `spent` the same way (77-90), so a run that pays 1,000 x 1-zat invoices passes with '1000 of 1000 zat spent' (re-executed on the real property)
- `crates/observation/src/lib.rs:234` — PayerRun::spent_zat() (doc line 233: 'Zatoshis actually spent under this grant') sums paid decision amounts only — the model has no field for the run's real outflow (the ledger summary at payer.rs:428-440 computes the same sum inline)
- `crates/probe/src/flow.rs:1544` — Invoice.amount_zat: u64 with no minimum, so 1-zat invoices (the maximum-fee-leverage shape) are valid invoices
- `crates/probe/src/ops.rs:59` — CONFIRM_BLOCKS = 11: every invoice also costs a mined-block round plus a wallet sync

**Dummy principal and affected resource:** the merchant issuing invoices: many tiny allowlisted invoices turn a 1,000-zat budget into ~10,000,000 zat of operator fees plus one broadcast and 11 mined blocks each

**Native input / invocation**

```
1,000 invoice files of 1 zat each to an allowlisted receiver under grant {budget_zat:1000, per_payment_cap_zat:400}
```
```
12,000,000 one-zat invoices under the demo grant defaults (budget 12,000,000 / cap 10,000,000 in scripts/bounded-payer-demo.sh)
```
- In the disposable sandbox copy, add crates/probe/src/verifier12_check.rs and `#[cfg(test)] mod verifier12_check;` to crates/probe/src/main.rs (the module only calls the target's own payer::decide and PayerPolicy::check; no production line changed), then run: cargo test --offline -p zanary-probe --bin zanary-probe -- verifier12 --nocapture --test-threads=1
- The same shape as the hunter's H3 run: cargo test --offline -p zanary-probe --bin zanary-probe -- hunter_payer --nocapture --test-threads=1
- Measuring the chain-level fee itself needs the regtest rig (out of sandbox bounds); this record does not claim that measurement

**Observed output:** Reproduced by the verifier in the sandbox (rust:1-slim, --network=none): the real payer::decide admitted exactly 1,000 payments of 1 zat under a 1,000-zat / 400-cap grant (spent 1,000) and refused the 1,001st with 'refused: paying 1 zat would take this run to 1001 of a 1000 zat budget'; the real payer-policy property PASSED a 1,000-row paid ledger with summary '1000 paid · 0 refused · 1000 of 1000 zat spent'; PayerRun::spent_zat() = 1,000. With the payer's own funding model (10,000 zat per payment, payer.rs:530, matching flow.rs:1170) that run tops up 10,000,000 zat of fees from outside the grant. The decision-layer result was executed; the fee magnitude is the payer's own model, not a chain observation.

**Conditions and containment:** the invoice directory holds many small invoices (e.g. 1 zat each) from a receiver on the allowlist, each within the per-payment cap; the grant's budget and cap are the run's only spend bounds; shortfall/top_up_for_target fund the fee headroom from outside the grant; each payment costs a ZIP-317 fee (10,000 zat per payment in the payer's own budget model) and one 11-block mining round

**Remediation:** denominate the budget in what the run actually spends: add the per-payment cost (ZIP-317 fee estimate) to the budget comparison in decide() and to payer-policy's re-derivation, and give the grant a bounded cardinality (a `max_payments`/`max_invoices` field or a minimum invoice amount) so a merchant cannot multiply the operator's fixed per-payment cost (fee + 11 mined blocks + a wallet sync) arbitrarily.

```rust
// crates/probe/src/payer.rs
/// ZIP-317 fee budget for one 2-action payment (the same per-payment figure shortfall() uses at 530).
pub const PAYMENT_FEE_ZAT: u64 = 10_000;
// in decide():
let after = spent_zat
    .saturating_add(invoice.amount_zat)
    .saturating_add(PAYMENT_FEE_ZAT);
if after > grant.budget_zat {
    return Decision { allowed: false, reason: format!(
        "refused: paying {} zat (+ {} zat fee) would take this run to {} of a {} zat budget",
        invoice.amount_zat, PAYMENT_FEE_ZAT, after, grant.budget_zat) };
}
// and bound the cardinality: e.g. refuse once `paid >= grant.max_payments`,
// with a `max_payments` field (serde default = unlimited) added to PayerGrant.
```
```rust
// crates/properties/src/properties/payer_policy.rs
// keep the detector in step with the rule it judges:
spent = spent
    .saturating_add(decision.amount_zat)
    .saturating_add(PAYMENT_FEE_ZAT);
// then the existing `if spent > grant.budget_zat { violations.push(...) }`
// fires for the fee-multiplied ledger, and a `paid > grant.max_payments`
// check covers the count bound.
```

## Invoice identity is a label, not a key: two invoice files carrying one invoice_id are both decided ALLOW and queued for payment in a single run, the run record puts the second payment's txid on the first row, and payer-policy passes the duplicated ledger

`zanary/payer-duplicate-invoice-id-replay` · overall **medium** (likelihood medium, impact medium) · confidence high

Nothing in the payer keys on invoice_id: load_invoices loads every *.json in the merchant-supplied directory (payer.rs:495-520; verifier-reproduced: two files carrying one id load as two invoices), decide() never consults it (84-136, it appears only inside reason text), the decision loop stores it as a label (262) and the repo has no dedupe, seen-set or uniqueness rule anywhere (grep over crates/). Two invoice files carrying the same invoice_id - a re-issued, corrected or re-sent invoice, which the merchant writes as an ordinary file ('issue an invoice the payer can act on with no channel other than the file', flow.rs:1554-1555) - are therefore two ordinary invoices: the production loop allows both while the cumulative budget holds and pushes both to to_pay, charging the budget twice (verifier-reproduced through the real CLI: two ALLOW lines for v13-8301, '300 of 1000' then '600 of 1000'; and with the loop's own statements: to_pay 2 entries, spent 600). The payment loop (291-363) pays every to_pay entry with no dedupe and the funding covers every entry including the duplicate (523-533), so the same invoice is paid twice in one run; the two shielded broadcasts themselves need the regtest rig and were not executed here. The attribution at 320-326 finds the FIRST row with that invoice_id, so the second payment's txid/height land on the first row while the second row is persisted with none (verifier-reproduced on the real PayerDecision type; a control with distinct ids attributes correctly) - ledger.jsonl, written per payment at 327-338, keeps its own two rows. payer-policy then passes the duplicated ledger because it has no notion of invoice identity (payer_policy.rs:63; verifier-reproduced: Pass, '2 paid - 0 refused - 600 of 1000 zat spent'), and the only property that reacts to this shape at all is amount-fingerprint's amount-repeat warning when the duplicate carries the same amount (it names equal amounts, never the repeated id, and fires only after both payments and the artifacts are written). The merchant is paid twice for one invoice and the run's payer ledger cannot be reconciled row-for-row against the chain.

**Trace**

1. *entrypoint* — `crates/probe/src/payer.rs:495` (load_invoices): every *.json entry in the merchant-supplied directory is parsed into an Invoice with no count cap and no invoice_id uniqueness check (503 selects by extension only, 511-518 parses); verifier-reproduced: two files carrying v13-8301 load as two invoices
1. *propagation* — `crates/probe/src/payer.rs:250` (decision loop): each invoice - including a second file repeating an id - is decided independently by decide(); the id is stored as a label at 262; verifier-reproduced through the real CLI: two ALLOW lines for v13-8301, budget 300 then 600 of 1000
1. *propagation* — `crates/probe/src/payer.rs:258` (to_pay): the allowed amount is added to `spent` and the invoice is pushed to to_pay (259), so a repeated id consumes the budget twice; verifier-reproduced with the loop's statements: to_pay 2 entries, spent 600, both decisions 'paid'
1. *propagation* — `crates/probe/src/payer.rs:291` (payment loop): every to_pay entry is proposed, signed and broadcast as its own shielded payment with no dedupe or per-entry guard; this is the unexecuted step (the wallet path needs the regtest rig) and it follows unconditionally from the loop plus the funding at 523-533
1. *sink* — `crates/probe/src/payer.rs:322` (txid attribution): `decisions.iter_mut().find(|d| d.invoice_id == invoice.invoice_id)` returns the FIRST row with that id, so the later payment's txid/height overwrite the earlier row and the later row keeps none; verifier-reproduced on the real PayerDecision type (control with distinct ids attributes correctly)

**Evidence**

- `crates/probe/src/payer.rs:262` — invoice_id is copied into each decision as a label; the file's only use of it as a lookup key is the find() at 320-326 (the other mentions are prints, the ledger row and the payment label)
- `crates/probe/src/payer.rs:322` — attribution by find() on invoice_id - the exact predicate; over two payments sharing v13-8301 it leaves row 0 holding the second txid/height and row 1 without any (verifier-reproduced)
- `crates/properties/src/properties/payer_policy.rs:63` — the property iterates decisions with no notion of invoice identity; the real check returns Pass for two paid rows sharing v13-8301 ('2 paid - 0 refused - 600 of 1000 zat spent', both rows echoed, one 'no txid')
- `crates/properties/src/properties/amount_fingerprint.rs:70` — the only property that reacts to this shape flags the equal amounts (W: 'exact repeat across payments'), never the repeated id; evaluated over the duplicated ledger it is the run's only failure, and the run would abort on it only after both payments and after the artifacts are written (payer.rs:406-411, 479-486)
- `crates/probe/src/payer.rs:247` — `let mut spent: u64 = 0;` - a fresh invocation re-decides the whole directory on a zero budget; the second identical CLI run printed the same 300 then 600 sequence
- `crates/probe/src/payer.rs:530` — the payer funds for every to_pay entry including a duplicate (10,000 zat ZIP-317 fee per entry; shortfall over to_pay, 523-533), so both payments are funded by design
- `crates/observation/src/lib.rs:224` — PayerRun holds label + grant + decisions only (224-230): no persisted set of decided or paid invoice ids to dedupe against across runs
- `crates/probe/src/flow.rs:1554` — the merchant side issues invoices with no channel other than the file ('issue an invoice the payer can act on with no channel other than the file', 1554-1555), so a re-issued invoice is a natural second file

**Dummy principal and affected resource:** the merchant, who writes the invoice files the payer is pointed at: it needs only the ability to place two *.json files in the --invoices directory. No access to the payer, its wallet or the grant is needed; the duplicate is an ordinary re-issued or re-sent invoice.

**Native input / invocation**

```
inv-a.json and inv-b.json, byte-identical: {"invoice_id":"v13-8301","amount_zat":300,"receiver":"uregtest1merchant","memo":"zanary: invoice v13-8301","uri":"zcash:?address=uregtest1merchant&amount=0.000003","issued_at_height":1,"merchant_account":0}
```
```
the same directory settled again by a second, identical invocation (fresh spent = 0; nothing consults the first run's ledger)
```
- rebuild the disposable copy of the reviewed revision (git archive HEAD | tar -x, rust-toolchain.toml removed) and add the verifier's module crates/probe/src/payer.rs#mod verifier13_checks; production lines 1-655 of that file stay byte-identical to the worktree (diff empty)
- docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <copy>:/t -v ~/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p zanary-probe --bin zanary-probe verifier13 -- --nocapture --test-threads=1'  (real load_invoices, the production loop's statements, the loop's attribution predicate, the real PayerPolicy::check and the full real property report; 5/5 passed)
- in the same sandbox: cargo build --offline -p zanary-probe --bin zanary-probe, then ./target/debug/zanary-probe payer --grant /t/v13-scratch/grant.json --invoices /t/v13-scratch/invoices --out /t/v13-scratch/out (grant: budget 1000, cap 400, allowlist [uregtest1merchant], expires 2030-01-01T00:00:00Z) - run it twice

**Observed output:** In the approved sandbox (--network=none, disposable copy, production lines byte-identical to the target): (a) real load_invoices reads 2 invoices, ids ["v13-8301","v13-8301"] (V13-1); (b) the real CLI payer over that directory prints ALLOW twice for v13-8301 - 'paid: within the cap (400 zat) and the budget (300 of 1000 zat spent after this)' then '600 of 1000' - then aborts at the node-RPC bootstrap (exit 1, --out never created); a second identical invocation prints the same 300 then 600 sequence (V13-CLI); (c) the production loop's statements queue both (to_pay 2 entries, spent 600, both decisions 'paid') (V13-2); (d) the loop's own attribution predicate over the real PayerDecision vector puts the second payment's txid/height on row 0 and leaves row 1 with none (control with distinct ids correct) (V13-3); (e) the real PayerPolicy::check returns Pass for the duplicated ledger ('2 paid - 0 refused - 600 of 1000 zat spent') (V13-4); (f) the full real property report over the duplicated ledger passes payer-policy, receiver-reuse, memo-absent, memo-content and the pool/policy properties, and FAILs only amount-fingerprint (W, 'exact repeat across payments' on the equal amounts) (V13-5). NOT executed: the two shielded broadcasts and the persisted run record of a real run - the payer's wallet path needs the regtest rig's funded synced wallet plus node/indexer, which is out of bounds; they are the unconditional continuation of the loop at 291-363 with funding for every entry (523-533).

**Conditions and containment:** two invoice JSON files in the --invoices directory carry the same invoice_id (a re-issued, corrected or re-sent invoice); each amount is within the per-payment cap and their sum within the budget; the payer is pointed at that directory; nothing rejects duplicates (no dedupe in intake, decide or the payment loop) and a second invocation starts from spent = 0, so the same directory is decided - and paid - again

**Remediation:** Treat invoice_id as a key in the authority model: refuse - and record - a second decision for an id already decided in the run, persist the decided/paid ids next to the ledger so a re-run over the same --out refuses ids already paid, and attribute txid/height by position (or a map keyed by (invoice_id, occurrence)) instead of find(), so a repeat cannot overwrite the first row.

```rust
// crates/probe/src/payer.rs
let mut decided: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
let mut to_pay: Vec<(usize, Invoice, String)> = Vec::new(); // decision index first
for (index, invoice) in invoices.iter().enumerate() {
    if !decided.insert(invoice.invoice_id.clone()) {
        decisions.push(PayerDecision {
            invoice_id: invoice.invoice_id.clone(),
            amount_zat: invoice.amount_zat,
            receiver: invoice.receiver.clone(),
            decision: "refused".to_string(),
            reason: format!("refused: invoice {} was already decided this run", invoice.invoice_id),
            txid: None,
            height: None,
        });
        continue;
    }
    // ... the existing decide()/to_pay path, now carrying `index` ...
}
// in the payment loop: for (index, (invoice, allowed_reason)) in to_pay.iter().enumerate() {
//     ... propose/sign/broadcast ...
//     decisions[*index].txid = Some(txid.to_string());
//     decisions[*index].height = height;
// }
// plus: before deciding, read the id set persisted next to --out and refuse ids already paid.
```

## Bounded payer enforces grant expiry as a text comparison, so an expiry instant written with a UTC offset (or an unparsable value) still pays

`zanary/payer-expiry-text-compare` · overall **medium** (likelihood medium, impact medium) · confidence high

decide() refuses an invoice only when the ASCII comparison `now >= grant.expires_at` holds (crates/probe/src/payer.rs:91). Both sides are unvalidated strings: `now` comes from the workspace clock (RFC3339 Z, fractional seconds) and a grant file's `expires_at` is used verbatim (payer.rs:173-184 parses only the *default* path, when the field is absent). Equal instants written differently therefore get opposite verdicts, and any text that sorts after the clock never expires. The lower-trust principal is whoever supplies the grant JSON to `zanary-probe payer --grant` (architecture.md boundary 3: caller-supplied JSON and paths into the probe CLI). No attacker is even needed: an operator or generic tool that writes the expiry from local time (a legal RFC3339 offset) or with a typo silently widens the authority window, and the control that should have caught it is the grant's own expiry rule ('the instant after which the authority is void', docs/BOUNDED-PAYER.md:31). The failure is invisible after the fact: PayerDecision records no decision time (observation/src/lib.rs:202-214) and payer-policy has no expiry rule (payer_policy.rs:114 only echoes the expiry; a paid row under a grant that expired in 2020 returns Pass).

**Trace**

1. *entrypoint* — `crates/probe/src/payer.rs:149` (zanary-probe payer --grant): the grant path is taken from argv (`args.require("--grant")`); the file is read at 165-166 and parsed into Grant at 167-168, so expires_at enters as a caller-supplied string
1. *propagation* — `crates/probe/src/payer.rs:173` (Grant -> PayerGrant): a file-supplied expires_at is used verbatim; only the absent (None) case parses and re-formats a timestamp (176-183)
1. *propagation* — `crates/probe/src/payer.rs:250` (decision loop): decide(&grant, spent, invoice, &now) is called once per invoice with the startup clock string (169-172)
1. *sink* — `crates/probe/src/payer.rs:91` (decide() expiry rule): lexicographic comparison: a past instant written with a positive offset sorts after the clock and is treated as unexpired; the invoice is then counted and pushed to to_pay (258-259) and paid by the loop at 291-308

**Evidence**

- `crates/probe/src/payer.rs:91` — `if now >= grant.expires_at.as_str()` — the expiry rule is a &str comparison with no parse on either side
- `crates/probe/src/payer.rs:173` — a grant file's expires_at is taken verbatim (`Some(ts) => ts`); the RFC3339 parser (178) runs only for the None case, and Grant (49-62) deserialises expires_at as an opaque Option<String> with no validator
- `crates/probe/src/record.rs:132` — utc_now_rfc3339 is the workspace clock and formats fractional seconds; data/runs/payer-pass/report.json carries verified_at "2026-10-16T10:57:34.300715717Z" while data/grants/*.json are whole-second Z
- `crates/observation/src/lib.rs:194` — PayerGrant.expires_at is a String (documented as an RFC3339 UTC instant) — the observation model never derives an instant from it
- `crates/observation/src/lib.rs:202` — PayerDecision carries invoice_id/amount/receiver/decision/reason/txid/height and no decision timestamp, so an expiry violation cannot be re-derived from the ledger
- `crates/properties/src/properties/payer_policy.rs:114` — payer-policy echoes grant.expires_at into its summary and evidence but never compares it to anything: the rules (78-102 cap/budget/allowlist/revoked, 68 the "error:" prefix) contain no expiry check, so a paid row under a long-expired grant is a Pass

**Dummy principal and affected resource:** whoever writes the grant JSON handed to `zanary-probe payer --grant` (architecture boundary 3: caller-supplied JSON and paths into the probe CLI); no privileged access is needed and no deliberate attacker is required — a local-time writer or one typo is enough

**Native input / invocation**

```
grant.json: {"budget_zat":1000,"per_payment_cap_zat":400,"allowlist":["uregtest1merchant"],"expires_at":"2026-11-02T13:00:00+14:00"} (the same instant as the canonical 2026-11-01T23:00:00Z the demo grants use)
```
```
grant.json: "expires_at": "never" (or any text sorting after the clock, e.g. "9999-01-01T00:00:00Z")
```
```
--now 2026-11-02T00:00:00Z with the two encodings of one instant: 2026-11-01T23:00:00Z (refused) vs 2026-11-02T13:00:00+14:00 (paid)
```
- git archive HEAD the reviewed revision into a disposable /t copy (the reviewed repo stays untouched); confirm crates/probe/src/payer.rs and payer_policy.rs match the repo md5 before editing (54d121450a61b6fb35395a40c94ee710, 0000a14d897c48f6fec7c39dd58929bc)
- append the bounded unit module (check-decide-src.txt) to /t/crates/probe/src/payer.rs
- docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 -v <sandbox>:/t -v <cargo registry>:ro -w /t rust:1-slim sh -c 'CARGO_BUILD_JOBS=6 cargo test --offline -p zanary-probe --bin zanary-probe -- verifier14 --nocapture --test-threads=1'
- cargo build --offline -p zanary-probe --bin zanary-probe, then run /t/target/debug/zanary-probe payer --grant /t/verifier14-cli/grants/grant-<canonical|offset|never>.json --invoices /t/verifier14-cli/invoices --now 2026-11-02T00:00:00Z for each grant
- append the payer-policy module (check-property-src.txt) and run cargo test --offline -p zanary-properties --lib -- verifier14 --nocapture --test-threads=1
- append the fail-closed fix module (check-fix-src.txt) and run the same probe-bin test command with filter verifier14_fixcheck

**Observed output:** Sandbox (rust:1-slim, --network=none, disposable git archive copy; no rig contact). (a) Unit, real decide(): 5/5 passed — now 2026-11-02T00:00:00Z with expires_at 2026-11-01T23:00:00Z -> allowed=false "refused: the grant expired at 2026-11-01T23:00:00Z (this invoice was shown at 2026-11-02T00:00:00Z)"; the SAME instant written 2026-11-02T13:00:00+14:00 -> allowed=true "paid: within the cap (400 zat) and the budget (10 of 1000 zat spent after this)" (the test asserts both strings parse to one instant and that the clock is at/past it); now 2026-11-01T00:00:00.300715717Z vs 2026-11-01T00:00:00Z -> allowed=true; "never"/"2030-99-99T00:00:00Z"/"9999-01-01T00:00:00Z" -> allowed=true while "" -> allowed=false (control: the rule is string ordering). (b) Real binary, real grant files: `zanary-probe payer --grant ... --invoices ... --now 2026-11-02T00:00:00Z` printed "REFUSE v14-8301 — refused: the grant expired at 2026-11-01T23:00:00Z ..." for the canonical grant and "ALLOW v14-8301 — paid: within the cap (400 zat) ..." for the offset and "never" grants, failing afterwards only at NodeRpc "Connection refused" because the sandbox has no network (decisions precede wallet use, payer.rs:246-271 vs 273-277). (c) Real payer-policy property: a paid row under a grant whose expires_at is 2020-01-01T00:00:00Z returns Pass, the expiry appearing only in the evidence text. Not exercised: the actual shielded broadcast (payer.rs:306-308), which needs the regtest rig.

**Conditions and containment:** the grant's expires_at is a valid RFC3339 instant written with a positive UTC offset (a local-time writer), whose literal text is later than its instant, so enforcement lapses for up to the offset window (<= ~14 h); or the value is unparsable text that sorts after the clock ('never'), which never expires at all; at the exact expiry second the payer's own clock text carries fractional seconds while a whole-second grant text is still greater, so the refusal lapses for the remainder of that second; the payer is invoked with that grant file; nothing rejects a non-canonical or malformed expires_at — no grant schema exists under schemas/, and payer.rs:91 is the only grant-expiry comparison in the probe crate

**Remediation:** Compare instants, not text: parse both `now` and the grant's expires_at with time::OffsetDateTime::parse (the crate already depends on `time` with `parsing`, payer.rs:176-179) and fail closed — refuse the invoices when expires_at is not a usable instant instead of reading it as unexpired. Additionally record a `decided_at` timestamp on every PayerDecision so the payer-policy property can re-derive the expiry rule after the fact (today PayerDecision has no decision time, observation/src/lib.rs:202-214). The snippet below keeps decide()'s Decision return type and was compile- and behaviour-checked in the sandbox: all four bad encodings refuse, a genuine future canonical expiry still pays.

```rust
// crates/probe/src/payer.rs
// decide(): compare instants, and fail closed when either side is not a usable instant.
let rfc3339 = time::format_description::well_known::Rfc3339;
let now_dt = match time::OffsetDateTime::parse(now, &rfc3339) {
    Ok(dt) => dt,
    Err(e) => {
        return Decision {
            allowed: false,
            reason: format!("refused: the clock {now} is not RFC3339: {e}"),
        }
    }
};
let expires_dt = match time::OffsetDateTime::parse(&grant.expires_at, &rfc3339) {
    Ok(dt) => dt,
    Err(e) => {
        return Decision {
            allowed: false,
            reason: format!(
                "refused: the grant's expires_at {} is not RFC3339 ({e}); refusing rather than treating it as unexpired",
                grant.expires_at
            ),
        }
    }
};
if now_dt >= expires_dt {
    return Decision {
        allowed: false,
        reason: format!(
            "refused: the grant expired at {} (this invoice was shown at {now})",
            grant.expires_at
        ),
    };
}
```

