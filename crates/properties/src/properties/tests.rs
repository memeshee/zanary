//! The proof suite: a `clean` fixture where every property passes, a `leaky` fixture where every
//! property fails, and one single-observation flip per property showing exactly which check
//! notices. Plus honesty tests: an empty run skips everything, and labels are never trusted.

use std::collections::BTreeMap;

use crate::{all_properties, evaluate, passes, Severity, Verdict};
use canary_observation::{FlowRun, Payment, PaymentKind, Pool};

use super::common::hex_of;
use super::fixtures;

fn verdicts(run: &FlowRun) -> BTreeMap<String, Verdict> {
    evaluate(run)
        .into_iter()
        .map(|f| (f.property, f.verdict))
        .collect()
}

fn failing_ids(run: &FlowRun) -> Vec<String> {
    let mut out: Vec<String> = evaluate(run)
        .into_iter()
        .filter(|f| f.verdict == Verdict::Fail)
        .map(|f| f.property)
        .collect();
    out.sort();
    out
}

fn verdict_of(run: &FlowRun, property: &str) -> Verdict {
    verdicts(run)
        .remove(property)
        .unwrap_or_else(|| panic!("no finding for {property}"))
}

fn assert_only_fails(run: &FlowRun, expected: &str) {
    assert_eq!(
        failing_ids(run),
        vec![expected.to_string()],
        "expected exactly {expected} to fail"
    );
}

// ---------------------------------------------------------------------------
// Registry + fixtures
// ---------------------------------------------------------------------------

#[test]
fn registry_has_all_twelve_in_report_order() {
    let props = all_properties();
    let ids: Vec<&str> = props.iter().map(|p| p.id()).collect();
    assert_eq!(
        ids,
        vec![
            "transparent-fallback",
            "unshielding",
            "receiver-reuse",
            "payer-linkability",
            "policy-downgrade",
            "memo-absent",
            "memo-content",
            "amount-fingerprint",
            "timing-correlation",
            "same-block-roundtrip",
            "server-metadata",
            "report-leak",
        ]
    );
    let severities: Vec<Severity> = props.iter().map(|p| p.severity()).collect();
    assert_eq!(&severities[..5], &[Severity::Disqualifying; 5]);
    assert_eq!(&severities[5..11], &[Severity::Warn; 6]);
    assert_eq!(severities[11], Severity::Note);
    // Report order: severity is non-increasing down the registry.
    for w in severities.windows(2) {
        assert!(w[0] >= w[1], "registry order violates severity ordering");
    }
}

#[test]
fn clean_fixture_passes_every_property() {
    let run = fixtures::clean_run();
    let findings = evaluate(&run);
    assert_eq!(findings.len(), 12);
    for f in &findings {
        assert_eq!(
            f.verdict,
            Verdict::Pass,
            "clean fixture: {} was {:?} (note: {:?}, evidence: {:?})",
            f.property,
            f.verdict,
            f.note,
            f.evidence
        );
    }
    assert!(passes(&findings), "a clean run must pass the D-gate");
}

#[test]
fn leaky_fixture_fails_every_property() {
    let run = fixtures::leaky_run();
    let findings = evaluate(&run);
    assert_eq!(findings.len(), 12);
    for f in &findings {
        assert_eq!(
            f.verdict,
            Verdict::Fail,
            "leaky fixture: {} was {:?} (note: {:?})",
            f.property,
            f.verdict,
            f.note
        );
        assert!(
            !f.evidence.is_empty(),
            "{} failed without evidence a stranger could re-check",
            f.property
        );
    }
    // Every D and every W fails; the D-gate closes.
    assert!(!passes(&findings));
}

#[test]
fn evaluate_orders_disqualifying_first() {
    let run = fixtures::leaky_run();
    let findings = evaluate(&run);
    assert!(findings[..5]
        .iter()
        .all(|f| f.severity == Severity::Disqualifying));
    assert!(findings[5..11].iter().all(|f| f.severity == Severity::Warn));
    assert_eq!(findings[11].severity, Severity::Note);
}

#[test]
fn empty_run_skips_everything_with_a_reason() {
    let run = FlowRun {
        flow_id: "empty".into(),
        flow_commit: "empty".into(),
        chain: "regtest".into(),
        started_at_height: 0,
        ended_at_height: 0,
        funding: vec![],
        payments: vec![],
        merchant_events: vec![],
        receipts: vec![],
        pool_totals: vec![],
        merchant_is_simulated: true,
    };
    let findings = evaluate(&run);
    assert_eq!(findings.len(), 12);
    for f in &findings {
        assert_eq!(
            f.verdict,
            Verdict::Skip,
            "empty run: {} should skip, was {:?}",
            f.property,
            f.verdict
        );
        assert!(
            f.note.is_some(),
            "{} skipped without a stated reason",
            f.property
        );
    }
}

#[test]
fn narration_is_not_trusted() {
    // Leaky bytes with clean-sounding labels: the checks still fire.
    let mut leaky = fixtures::leaky_run();
    for (i, p) in leaky.payments.iter_mut().enumerate() {
        p.label = format!("fully-shielded private payment {i}");
    }
    assert_eq!(verdict_of(&leaky, "transparent-fallback"), Verdict::Fail);
    assert_eq!(verdict_of(&leaky, "unshielding"), Verdict::Fail);

    // Clean bytes with scary-sounding labels: the checks still pass.
    let mut clean = fixtures::clean_run();
    for (i, p) in clean.payments.iter_mut().enumerate() {
        p.label = format!("transparent fallback path {i}");
    }
    assert_eq!(failing_ids(&clean), Vec::<String>::new());
}

// ---------------------------------------------------------------------------
// Per-property flip tests: one observation changed, exactly one check notices.
// ---------------------------------------------------------------------------

#[test]
fn flip_transparent_fallback_recipient() {
    let mut run = fixtures::clean_run();
    run.payments[0].recipient_receivers =
        vec!["tmFlipTransparentRecipient111111111111111111".to_string()];
    assert_only_fails(&run, "transparent-fallback");
}

#[test]
fn flip_unshielding_pool_totals_delta() {
    let mut run = fixtures::clean_run();
    run.pool_totals.push(canary_observation::PoolTotals {
        height: 117,
        transparent_zat: 0,
        sapling_zat: 0,
        orchard_zat: 500_000_000,
    });
    run.pool_totals.push(canary_observation::PoolTotals {
        height: 118,
        transparent_zat: 5_000_000,
        sapling_zat: 0,
        orchard_zat: 495_000_000,
    });
    assert_only_fails(&run, "unshielding");
}

#[test]
fn flip_receiver_reuse() {
    let mut run = fixtures::clean_run();
    let shared = run.payments[0].recipient_receivers.clone();
    run.payments[1].recipient_receivers = shared;
    assert_only_fails(&run, "receiver-reuse");
}

#[test]
fn flip_payer_linkability_sender_receivers() {
    let mut run = fixtures::clean_run();
    let sender = run.funding[0].receiver.clone();
    run.payments[0].sender_receivers = vec![sender.clone()];
    run.payments[1].sender_receivers = vec![sender];
    assert_only_fails(&run, "payer-linkability");
}

/// A batched settlement: two invoices in one transaction is public proof they were created
/// together, so the property that measures payer-linkability must fail — and fail on *that*
/// signal rather than on a heuristic happening to cross the threshold.
#[test]
fn flip_payer_linkability_shared_transaction() {
    let mut run = fixtures::clean_run();
    let batch = run.payments[0].txid.clone();
    run.payments[1].txid = batch.clone();
    run.payments[1].height = run.payments[0].height;
    // Marked the way the flow runner marks a batch: the run records the batch on every payment.
    for payment in run.payments.iter_mut().take(2) {
        payment
            .observed
            .insert("batch_id".to_string(), batch.clone());
        payment
            .observed
            .insert("batch_size".to_string(), "2".to_string());
    }
    let findings = crate::evaluate(&run);
    let finding = findings
        .iter()
        .find(|f| f.property == "payer-linkability")
        .expect("payer-linkability must be evaluated");
    assert_eq!(finding.verdict, Verdict::Fail);
    assert!(
        finding
            .evidence
            .iter()
            .any(|line| line.contains("shared_transaction(1.00)")),
        "the batch must be linked by the shared-transaction signal, not by a heuristic: {:?}",
        finding.evidence
    );
    assert_only_fails(&run, "payer-linkability");
}

#[test]
fn flip_policy_downgrade_argument() {
    let mut run = fixtures::clean_run();
    run.payments[0].policy = Some("AllowRevealedAmounts".to_string());
    assert_only_fails(&run, "policy-downgrade");
}

#[test]
fn flip_memo_absent_removes_binding() {
    let mut run = fixtures::clean_run();
    run.payments[0].memo_hex = None;
    run.payments[0].memo_text = None;
    assert_only_fails(&run, "memo-absent");
}

#[test]
fn flip_memo_content_adds_pii() {
    let mut run = fixtures::clean_run();
    let memo = "alice@example.com INV-1001";
    run.payments[0].memo_text = Some(memo.to_string());
    run.payments[0].memo_hex = Some(hex_of(memo));
    assert_only_fails(&run, "memo-content");
}

#[test]
fn flip_amount_fingerprint_echoes_funding() {
    let mut run = fixtures::clean_run();
    run.payments[0].amount_zat = run.funding[0].amount_zat;
    assert_only_fails(&run, "amount-fingerprint");
}

#[test]
fn flip_timing_correlation_adjacent_block() {
    let mut run = fixtures::clean_run();
    run.payments[0].height = Some(101);
    assert_only_fails(&run, "timing-correlation");
}

#[test]
fn flip_same_block_roundtrip_funding_height() {
    let mut run = fixtures::clean_run();
    run.funding[0].height = Some(110);
    assert_only_fails(&run, "same-block-roundtrip");
}

#[test]
fn flip_server_metadata_identity_field() {
    let mut run = fixtures::clean_run();
    run.merchant_events[0]
        .fields
        .insert("email".to_string(), "alice@example.com".to_string());
    assert_only_fails(&run, "server-metadata");
}

#[test]
fn flip_report_leak_disclosure() {
    let mut run = fixtures::clean_run();
    run.receipts[0].discloses = vec!["viewing_key".to_string()];
    assert_only_fails(&run, "report-leak");
}

// ---------------------------------------------------------------------------
// Boundary and honesty edge cases
// ---------------------------------------------------------------------------

#[test]
fn policy_negation_is_not_a_downgrade() {
    let mut run = fixtures::clean_run();
    run.payments[0].policy = Some("NoAllowRevealedAmounts".to_string());
    assert_eq!(failing_ids(&run), Vec::<String>::new());
    assert_eq!(verdict_of(&run, "policy-downgrade"), Verdict::Pass);
}

#[test]
fn timing_window_boundary_is_two_blocks() {
    let mut run = fixtures::clean_run();
    run.payments[0].height = Some(102); // delay 2 > window 1
    assert_eq!(failing_ids(&run), Vec::<String>::new());
    assert_eq!(verdict_of(&run, "timing-correlation"), Verdict::Pass);
}

#[test]
fn amount_epsilon_boundary() {
    // diff == epsilon with a single funding candidate -> flagged.
    let mut run = fixtures::clean_run();
    run.payments[0].amount_zat =
        run.funding[0].amount_zat - super::amount_fingerprint::AMOUNT_EPSILON_ZAT;
    assert_only_fails(&run, "amount-fingerprint");

    // diff == epsilon + 1 -> not flagged.
    let mut run = fixtures::clean_run();
    run.payments[0].amount_zat =
        run.funding[0].amount_zat - super::amount_fingerprint::AMOUNT_EPSILON_ZAT - 1;
    assert_eq!(failing_ids(&run), Vec::<String>::new());
}

#[test]
fn receiver_reuse_skips_with_a_single_payment() {
    let mut run = fixtures::clean_run();
    run.payments.truncate(1);
    assert_eq!(verdict_of(&run, "receiver-reuse"), Verdict::Skip);
}

#[test]
fn unshielding_skips_when_pool_direction_is_unknown() {
    let mut run = fixtures::clean_run();
    run.pool_totals.clear();
    run.payments = vec![Payment {
        label: "opaque".into(),
        txid: fixtures::tid(0x51),
        height: Some(110),
        amount_zat: 1_234_567,
        recipient_receivers: vec![],
        sender_receivers: vec![],
        memo_hex: None,
        memo_text: None,
        invoice_id: None,
        pool_in: Pool::Unknown,
        pool_out: Pool::Unknown,
        kind: PaymentKind::Unknown,
        policy: None,
        observed: BTreeMap::new(),
    }];
    assert_eq!(verdict_of(&run, "unshielding"), Verdict::Skip);
}

#[test]
fn memo_absent_requires_the_invoice_inside_the_memo() {
    let mut run = fixtures::clean_run();
    let memo = "unrelated text";
    run.payments[0].memo_text = Some(memo.to_string());
    run.payments[0].memo_hex = Some(hex_of(memo));
    assert_only_fails(&run, "memo-absent");
}

#[test]
fn server_metadata_flags_ip_linkage_between_two_payments() {
    let mut run = fixtures::clean_run();
    run.merchant_events[0].remote_ip = Some("203.0.113.9".to_string());
    run.merchant_events[1].remote_ip = Some("203.0.113.9".to_string());
    assert_only_fails(&run, "server-metadata");
}

#[test]
fn report_leak_flags_key_material_in_body() {
    let mut run = fixtures::clean_run();
    run.receipts[0].body.push_str(" ivk: uvk1abc");
    assert_only_fails(&run, "report-leak");
}
