//! `unshielding` (D) — value leaves the shielded pool, or the flow routes through transparent.
//!
//! Leak: the moment funds leave the pool the payout is public; a merchant that unshields daily
//! has published its own revenue.
//!
//! Detect: three independent signals —
//!  1. a payment whose observed `pool_out` is `Transparent` (direct unshield), with pool totals
//!     before/after when snapshots exist;
//!  2. chain pool-total movement between consecutive snapshots where the shielded pools
//!     (sapling + orchard) decrease and transparent increases (the transaction may or may not be
//!     attributed in the record);
//!  3. routes: a later payment spending from transparent (`pool_in == Transparent`) after an
//!     earlier payment that ended transparent or in a mixed bundle.
//!
//! Evidence: the txid that moved value out of the pool, with before/after pool totals.

use crate::{Finding, Property, Severity};
use canary_observation::{FlowRun, Payment, PaymentKind, Pool, PoolTotals};

use super::common::{height_str, pool_str};

pub struct Unshielding;

fn shielded_zat(t: &PoolTotals) -> u128 {
    t.sapling_zat as u128 + t.orchard_zat as u128
}

/// "h=100 transparent=0 sapling=0 orchard=500000000" style line for one snapshot.
fn totals_line(t: &PoolTotals) -> String {
    format!(
        "h={} transparent={} sapling={} orchard={}",
        t.height, t.transparent_zat, t.sapling_zat, t.orchard_zat
    )
}

fn totals_context(totals: &[PoolTotals], height: Option<u32>) -> Option<String> {
    let h = height?;
    let before = totals
        .iter()
        .filter(|t| t.height <= h)
        .max_by_key(|t| t.height)?;
    match totals
        .iter()
        .filter(|t| t.height > h)
        .min_by_key(|t| t.height)
    {
        Some(after) => Some(format!(
            "pool_totals {} → {}",
            totals_line(before),
            totals_line(after)
        )),
        None => Some(format!(
            "pool_totals {} (no later snapshot to compare)",
            totals_line(before)
        )),
    }
}

impl Property for Unshielding {
    fn id(&self) -> &'static str {
        "unshielding"
    }

    fn severity(&self) -> Severity {
        Severity::Disqualifying
    }

    fn title(&self) -> &'static str {
        "no value leaves the shielded pool"
    }

    fn check(&self, run: &FlowRun) -> Finding {
        let mut evidence: Vec<String> = Vec::new();

        // Signal 1: a payment that ended transparent.
        for p in &run.payments {
            if p.pool_out == Pool::Transparent {
                evidence.push(format!(
                    "txid={} height={} pool_in={} pool_out=transparent amount_zat={} (value left the shielded pool)",
                    p.txid,
                    height_str(p.height),
                    pool_str(p.pool_in),
                    p.amount_zat
                ));
                if let Some(ctx) = totals_context(&run.pool_totals, p.height) {
                    evidence.push(ctx);
                }
            }
        }

        // Signal 2: pool-total movement, attributed when possible.
        let mut totals: Vec<&PoolTotals> = run.pool_totals.iter().collect();
        totals.sort_by_key(|t| t.height);
        for w in totals.windows(2) {
            let (a, b) = (w[0], w[1]);
            let shielded_a = shielded_zat(a);
            let shielded_b = shielded_zat(b);
            // A shielded payment pays its ZIP-317 fee out of the shielded pool, and that fee
            // lands in the miner's coinbase: shielded value does fall while transparent value
            // rises, by exactly the fees of the payments in this interval. That is a property of
            // every Zcash transaction, not value the flow routed through transparent, so a
            // decrease the run's own recorded fees explain is not a finding. The runner records
            // each payment's fee as `observed.fee_zat`; a decrease beyond that is unexplained and
            // still counts.
            let explained_fees_zat: u128 = run
                .payments
                .iter()
                .filter(|p| {
                    p.height
                        .map(|h| h > a.height && h <= b.height)
                        .unwrap_or(false)
                })
                .filter_map(|p| p.observed.get("fee_zat"))
                .filter_map(|fee| fee.parse::<u128>().ok())
                .sum();
            let unexplained_zat = shielded_a
                .saturating_sub(shielded_b)
                .saturating_sub(explained_fees_zat);
            if unexplained_zat > 0 && b.transparent_zat > a.transparent_zat {
                evidence.push(format!(
                    "pool_totals h={} shielded={} → h={} shielded={}, transparent {} → {} (shielded value left the pool between these snapshots)",
                    a.height, shielded_a, b.height, shielded_b, a.transparent_zat, b.transparent_zat
                ));
                let at: Vec<&Payment> = run
                    .payments
                    .iter()
                    .filter(|p| p.height == Some(b.height))
                    .collect();
                if at.is_empty() {
                    evidence.push(format!(
                        "no payment recorded at height={} to attribute the movement (height recorded from pool snapshots only)",
                        b.height
                    ));
                } else {
                    for p in at {
                        evidence.push(format!(
                            "attributed txid={} height={} pool_in={} pool_out={}",
                            p.txid,
                            height_str(p.height),
                            pool_str(p.pool_in),
                            pool_str(p.pool_out)
                        ));
                    }
                }
            }
        }

        // Signal 3: routes — a transparent hop between shielded legs.
        let mut ordered: Vec<(usize, &Payment)> = run.payments.iter().enumerate().collect();
        ordered.sort_by_key(|(i, p)| (p.height.unwrap_or(u32::MAX), *i));
        for (idx, (_, b)) in ordered.iter().enumerate() {
            if b.pool_in != Pool::Transparent {
                continue;
            }
            for (_, a) in ordered[..idx].iter().rev() {
                if a.pool_out == Pool::Transparent {
                    evidence.push(format!(
                        "route txid={} pool_out=transparent → txid={} pool_in=transparent (value re-entered the pool through a transparent hop)",
                        a.txid, b.txid
                    ));
                    break;
                }
                if a.pool_out == Pool::CrossPool {
                    evidence.push(format!(
                        "possible route txid={} pool_out=cross_pool → txid={} pool_in=transparent (a mixed bundle may have unshielded value)",
                        a.txid, b.txid
                    ));
                    break;
                }
            }
        }

        if !evidence.is_empty() {
            return Finding::fail(
                "unshielding",
                Severity::Disqualifying,
                self.title(),
                evidence,
            )
            .with_note(
                "signals: direct transparent pool_out, shielded→transparent movement between \
                     pool snapshots (chain-wide; the throwaway chain has no other actors), and \
                     transparent hops between shielded legs. A shield (transparent in, shielded \
                     out) is not flagged — entering the pool is not leaving it.",
            );
        }

        if run.payments.is_empty() {
            return Finding::skip(
                "unshielding",
                Severity::Disqualifying,
                self.title(),
                "no payments recorded; there is no spending side to trace value out of the pool",
            );
        }

        let undecidable = run.pool_totals.is_empty()
            && run.payments.iter().any(|p| {
                matches!(p.pool_out, Pool::Unknown | Pool::CrossPool)
                    || matches!(p.pool_in, Pool::Unknown)
                    || matches!(p.kind, PaymentKind::Unknown)
            });
        if undecidable {
            return Finding::skip(
                "unshielding",
                Severity::Disqualifying,
                self.title(),
                "at least one payment has an unknown/mixed pool direction and no pool-total \
                 snapshots were recorded, so no value movement can be traced",
            );
        }

        Finding::pass("unshielding", Severity::Disqualifying, self.title()).with_note(format!(
            "{} payments checked; no transparent pool_out, no shielded→transparent movement across \
             {} pool snapshots, no transparent hop between shielded legs",
            run.payments.len(),
            run.pool_totals.len()
        ))
    }
}
