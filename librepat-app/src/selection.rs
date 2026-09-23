use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(crate) enum RangeSelection {
    Replace,
    Add,
    Remove,
}

pub(crate) fn apply_range_selection(
    selection: &mut BTreeSet<usize>,
    ordered_indices: &[usize],
    anchor: Option<usize>,
    target: usize,
    action: RangeSelection,
) -> usize {
    let anchor_position = anchor
        .and_then(|anchor| position(ordered_indices, anchor).map(|position| (anchor, position)));
    let (range, effective_anchor) = anchor_position
        .zip(position(ordered_indices, target))
        .map(|((anchor, anchor_position), target_position)| {
            let (start, end) = if anchor_position <= target_position {
                (anchor_position, target_position)
            } else {
                (target_position, anchor_position)
            };
            (&ordered_indices[start..=end], anchor)
        })
        .unwrap_or_else(|| (std::slice::from_ref(&target), target));

    match action {
        RangeSelection::Replace => {
            selection.clear();
            selection.extend(range.iter().copied());
        }
        RangeSelection::Add => selection.extend(range.iter().copied()),
        RangeSelection::Remove => {
            for index in range {
                selection.remove(index);
            }
        }
    }
    effective_anchor
}

fn position(indices: &[usize], target: usize) -> Option<usize> {
    indices.iter().position(|&index| index == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_range_selection_should_follow_visible_order_inclusively() {
        let mut selection = BTreeSet::from([8]);

        apply_range_selection(
            &mut selection,
            &[4, 2, 8, 1],
            Some(4),
            8,
            RangeSelection::Replace,
        );

        assert_eq!(selection, BTreeSet::from([2, 4, 8]));
    }

    #[test]
    fn apply_range_selection_should_support_reverse_ranges() {
        let mut selection = BTreeSet::new();

        apply_range_selection(
            &mut selection,
            &[4, 2, 8, 1],
            Some(8),
            4,
            RangeSelection::Replace,
        );

        assert_eq!(selection, BTreeSet::from([2, 4, 8]));
    }

    #[test]
    fn apply_range_selection_should_add_to_an_existing_selection() {
        let mut selection = BTreeSet::from([9]);

        apply_range_selection(
            &mut selection,
            &[4, 2, 8, 1],
            Some(2),
            1,
            RangeSelection::Add,
        );

        assert_eq!(selection, BTreeSet::from([1, 2, 8, 9]));
    }

    #[test]
    fn apply_range_selection_should_remove_checked_range() {
        let mut selection = BTreeSet::from([1, 2, 4, 8, 9]);

        apply_range_selection(
            &mut selection,
            &[4, 2, 8, 1],
            Some(2),
            1,
            RangeSelection::Remove,
        );

        assert_eq!(selection, BTreeSet::from([4, 9]));
    }

    #[test]
    fn apply_range_selection_should_use_target_when_anchor_is_hidden() {
        let mut selection = BTreeSet::from([4]);

        let anchor = apply_range_selection(
            &mut selection,
            &[2, 8, 1],
            Some(4),
            8,
            RangeSelection::Replace,
        );

        assert_eq!((selection, anchor), (BTreeSet::from([8]), 8));
    }
}
