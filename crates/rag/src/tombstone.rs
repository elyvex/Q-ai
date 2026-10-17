//! Tombstone status propagation helpers.

/// Returns true if a source status allows its documents to be queried.
pub fn is_queryable(status: &str) -> bool {
    status == "active"
}

/// Returns true if deactivation/tombstoning must propagate deletes to derived index entries.
pub fn propagates(status: &str) -> bool {
    status == "deactivated" || status == "tombstoned"
}

/// Returns true if the status is terminal (no reactivation).
pub fn is_terminal(status: &str) -> bool {
    status == "tombstoned"
}

/// Returns true if reactivation to active is allowed from this status.
pub fn allows_reactivation(status: &str) -> bool {
    status == "deactivated"
}