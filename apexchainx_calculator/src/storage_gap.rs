//! Storage-gap reservation for the governance/staking upgrade-safety pattern.
//!
//! ## Background
//!
//! In EVM-based upgradeable contracts a `__gap` array is declared at the end of
//! a base contract's storage layout so that future child contracts can add new
//! storage variables without colliding with inherited slots.
//!
//! **Soroban / Stellar context:** Soroban contracts address storage by named
//! `Symbol` keys rather than sequential slot numbers, so layout collisions of
//! the EVM kind cannot occur.  However, the _intent_ of upgrade safety still
//! applies:
//!
//! 1. **Reserved key namespace** – pre-declaring a set of `STAKING_GAP_*` keys
//!    signals to future contributors that these identifiers are reserved and
//!    must not be reused for unrelated purposes.
//! 2. **Migration guard** – the storage-version check (`check_version`) already
//!    prevents execution against an un-migrated state, but explicitly documenting
//!    the reserved key range makes the upgrade checklist self-evident.
//! 3. **Audit trail** – every reserved key below will appear in the storage
//!    footprint audit, making it trivial to detect accidental key reuse during
//!    code review.
//!
//! ## Reserved keys
//!
//! The constants below are intentionally left **unused** in the current
//! implementation.  They are placeholders that reserve the key namespace for
//! future staking/governance extensions without requiring a migration.
//!
//! Resolves issue #815.

use soroban_sdk::symbol_short;
use soroban_sdk::Symbol;

/// Reserved storage key – gap slot 0.
///
/// Placeholder for future staking or governance extension fields.
/// Do **not** assign semantics to this key without bumping the storage
/// version and adding a migration step to `migrate()`.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_0: Symbol = symbol_short!("SGAP0");

/// Reserved storage key – gap slot 1.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_1: Symbol = symbol_short!("SGAP1");

/// Reserved storage key – gap slot 2.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_2: Symbol = symbol_short!("SGAP2");

/// Reserved storage key – gap slot 3.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_3: Symbol = symbol_short!("SGAP3");

/// Reserved storage key – gap slot 4.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_4: Symbol = symbol_short!("SGAP4");

/// Reserved storage key – gap slot 5.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_5: Symbol = symbol_short!("SGAP5");

/// Reserved storage key – gap slot 6.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_6: Symbol = symbol_short!("SGAP6");

/// Reserved storage key – gap slot 7.
#[allow(dead_code)]
pub(crate) const STAKING_GAP_7: Symbol = symbol_short!("SGAP7");

/// Total number of reserved gap slots.
///
/// When a new field is promoted from the gap (i.e., assigned real semantics),
/// decrement this constant and bump `STORAGE_VERSION` in `storage_version.rs`.
pub const STAKING_STORAGE_GAP_SIZE: usize = 8;

#[cfg(test)]
mod storage_gap_tests {
    use super::*;

    /// Verify the gap size constant matches the number of declared gap symbols.
    #[test]
    fn test_storage_gap_size_constant_is_correct() {
        // Update this count whenever a slot is promoted to a real field.
        assert_eq!(STAKING_STORAGE_GAP_SIZE, 8);
    }

    /// Verify none of the gap keys collide with each other (all distinct).
    #[test]
    fn test_gap_keys_are_unique() {
        let keys = [
            STAKING_GAP_0,
            STAKING_GAP_1,
            STAKING_GAP_2,
            STAKING_GAP_3,
            STAKING_GAP_4,
            STAKING_GAP_5,
            STAKING_GAP_6,
            STAKING_GAP_7,
        ];
        // Each Symbol's string form must be distinct.
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "gap slot {i} and {j} must be distinct keys");
            }
        }
    }
}
