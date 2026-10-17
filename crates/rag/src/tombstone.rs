//! Tombstone status propagation helpers.

/// Returns true if a source status allows its documents to be queried.
pub fn is_queryable(status: &str) -> bool {
    status == "active"
}
