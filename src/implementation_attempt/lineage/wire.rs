//! Wire names of Kiln's reasons, framed into the binding identity.

impl super::Blocker {
    pub(super) const fn wire(self) -> &'static str {
        match self {
            Self::Denied => "denied",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
            Self::BudgetExhausted => "budget_exhausted",
        }
    }
}

impl super::Backpressure {
    pub(super) const fn wire(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Eligible => "eligible",
            Self::Waiting => "waiting",
            Self::Overloaded => "overloaded",
            Self::Stale => "stale",
            Self::Incomplete => "incomplete",
            Self::Incompatible => "incompatible",
        }
    }
}
