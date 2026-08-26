use std::collections::HashSet;

use crate::TestResult;

/// Returns the final occurrence of each imported test kind in source order.
#[must_use]
pub fn latest_test_indices(tests: &[TestResult]) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut indices = tests
        .iter()
        .enumerate()
        .rev()
        .filter_map(|(index, test)| seen.insert(&test.kind).then_some(index))
        .collect::<Vec<_>>();
    indices.reverse();
    indices
}

/// Returns the final occurrence of each imported test kind in source order.
#[must_use]
pub fn latest_tests(tests: &[TestResult]) -> Vec<&TestResult> {
    latest_test_indices(tests)
        .into_iter()
        .map(|index| &tests[index])
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{TestKind, TestResult, TestStatus};

    use super::{latest_test_indices, latest_tests};

    fn result(kind: TestKind, status: TestStatus) -> TestResult {
        TestResult {
            raw_name: format!("{kind:?}"),
            kind,
            status,
            measurement: None,
            limit: None,
            raw_line: String::new(),
        }
    }

    #[test]
    fn latest_tests_should_keep_only_the_final_occurrence_of_each_kind() {
        let tests = [
            result(TestKind::EarthBond, TestStatus::Fail),
            result(TestKind::Insulation, TestStatus::Pass),
            result(TestKind::EarthBond, TestStatus::Pass),
        ];

        assert_eq!(latest_test_indices(&tests), [1, 2]);
        assert_eq!(
            latest_tests(&tests)
                .into_iter()
                .map(|test| test.status)
                .collect::<Vec<_>>(),
            [TestStatus::Pass, TestStatus::Pass]
        );
    }
}
