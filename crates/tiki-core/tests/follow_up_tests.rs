mod support;

use support::TestDay;
use tiki_core::TikiSettings;
use tiki_core::domain::FollowUpPolicy;

#[test]
fn the_follow_up_threshold_and_the_first_nudge_come_from_one_policy() {
    let policy = FollowUpPolicy::new(&TikiSettings::default());

    assert!(!policy.is_due(2));
    assert!(policy.is_due(3));
    assert_eq!(
        policy.first_nudge(TestDay::october(18)),
        Some(TestDay::october(21).and_hms_opt(9, 0, 0).unwrap())
    );
}
