//! Pure menu bar active-index reducers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActiveIndexRequest {
    pub active: usize,
    pub menu_count: usize,
}

pub(super) fn clamp_active_index(request: ActiveIndexRequest) -> Option<usize> {
    let last_index = request.menu_count.checked_sub(1)?;
    Some(request.active.min(last_index))
}

pub(super) fn previous_active_index(request: ActiveIndexRequest) -> Option<usize> {
    let last_index = request.menu_count.checked_sub(1)?;
    if request.active == 0 {
        return Some(last_index);
    }
    Some(request.active.saturating_sub(1))
}

pub(super) fn next_active_index(request: ActiveIndexRequest) -> Option<usize> {
    let divisor = request.menu_count;
    if divisor == 0 {
        return None;
    }
    debug_assert!(divisor != 0);
    Some(request.active.saturating_add(1) % divisor)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVE_INDEX: usize = 1;
    const MENU_COUNT: usize = 3;
    const OUT_OF_RANGE_ACTIVE_INDEX: usize = 9;
    const LAST_INDEX: usize = 2;
    const NEXT_INDEX: usize = 2;

    #[test]
    fn clamps_active_index_to_last_menu() {
        let request = ActiveIndexRequest {
            active: OUT_OF_RANGE_ACTIVE_INDEX,
            menu_count: MENU_COUNT,
        };

        assert_eq!(clamp_active_index(request), Some(LAST_INDEX));
    }

    #[test]
    fn wraps_previous_and_next_indices() {
        let first = ActiveIndexRequest {
            active: 0,
            menu_count: MENU_COUNT,
        };
        let middle = ActiveIndexRequest {
            active: ACTIVE_INDEX,
            menu_count: MENU_COUNT,
        };

        assert_eq!(previous_active_index(first), Some(LAST_INDEX));
        assert_eq!(next_active_index(middle), Some(NEXT_INDEX));
    }

    #[test]
    fn empty_menu_has_no_active_index() {
        let request = ActiveIndexRequest {
            active: ACTIVE_INDEX,
            menu_count: 0,
        };

        assert_eq!(clamp_active_index(request), None);
        assert_eq!(previous_active_index(request), None);
        assert_eq!(next_active_index(request), None);
    }
}
