//! The 12 leak-class properties, one module each, plus the registry that returns them in report
//! order (disqualifying first). Each property is a pure function over a recorded
//! [`zanary_observation::FlowRun`].

pub(crate) mod common;

pub mod amount_fingerprint;
pub mod memo_absent;
pub mod memo_content;
pub mod payer_linkability;
pub mod policy_downgrade;
pub mod receiver_reuse;
pub mod report_leak;
pub mod same_block_roundtrip;
pub mod server_metadata;
pub mod timing_correlation;
pub mod transparent_fallback;
pub mod unshielding;

pub use amount_fingerprint::{AMOUNT_EPSILON_ZAT, ROUND_GRANULARITY_ZAT};
pub use payer_linkability::CONFIDENCE_THRESHOLD as PAYER_LINKABILITY_THRESHOLD;
pub use timing_correlation::TIMING_WINDOW_BLOCKS;

use crate::Property;

/// Every property, in report order: the five disqualifying classes first, then warnings, then
/// notes — the order of the table in `docs/PROPERTIES.md`.
pub(crate) fn registry() -> Vec<Box<dyn Property>> {
    vec![
        Box::new(transparent_fallback::TransparentFallback),
        Box::new(unshielding::Unshielding),
        Box::new(receiver_reuse::ReceiverReuse),
        Box::new(payer_linkability::PayerLinkability),
        Box::new(policy_downgrade::PolicyDowngrade),
        Box::new(memo_absent::MemoAbsent),
        Box::new(memo_content::MemoContent),
        Box::new(amount_fingerprint::AmountFingerprint),
        Box::new(timing_correlation::TimingCorrelation),
        Box::new(same_block_roundtrip::SameBlockRoundtrip),
        Box::new(server_metadata::ServerMetadata),
        Box::new(report_leak::ReportLeak),
    ]
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
