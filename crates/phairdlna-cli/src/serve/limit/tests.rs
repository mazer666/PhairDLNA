//! Tests for the daemon's shared connection budget.

use super::ConnectionBudget;

/// The budget lets the connections in until the bound is reached.
#[test]
fn the_budget_counts_the_connections() {
    let budget = ConnectionBudget::new(2);

    assert!(budget.admits());
    assert!(budget.admits());
    assert!(!budget.admits(), "the third connection is refused");
    assert_eq!(budget.open(), 2);
}

/// A place that is given back lets the next connection in.
#[test]
fn a_place_is_given_back() {
    let budget = ConnectionBudget::new(1);

    assert!(budget.admits());
    assert!(!budget.admits());
    budget.release();
    assert!(budget.admits(), "the place of the connection is free again");
}
