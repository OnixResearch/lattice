/// Canonical mutating-tool kind used by provenance checkpoint policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceMutationKind {
    /// Existing content was edited in place.
    Edit,
    /// Content was written or created.
    Write,
    /// Content was replaced as a whole.
    Replace,
    /// A file identity moved from one relative path to another.
    Rename,
}

impl ProvenanceMutationKind {
    /// Parses a canonical mutation kind label.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when the label is empty, unsafe, or not one
    /// of the supported canonical mutation kinds.
    pub fn parse(value: &str) -> Result<Self, ProvenanceError> {
        let value = validate_safe_text("mutation_kind", value.to_owned())?;
        match value.as_str() {
            "edit" => Ok(Self::Edit),
            "write" => Ok(Self::Write),
            "replace" => Ok(Self::Replace),
            "rename" => Ok(Self::Rename),
            other => Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidPolicy,
                "mutation_kind",
                other,
                "unknown canonical mutation kind",
            )),
        }
    }

    /// Returns the canonical mutation kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::Write => "write",
            Self::Replace => "replace",
            Self::Rename => "rename",
        }
    }
}

/// Inclusive one-based line range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProvenanceLineRange {
    start: u32,
    end: u32,
}

impl ProvenanceLineRange {
    /// Builds a validated inclusive line range.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when line numbers are zero or reversed.
    pub fn new(start: u32, end: u32) -> Result<Self, ProvenanceError> {
        if start < MIN_LINE_NUMBER || end < MIN_LINE_NUMBER {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidRange,
                "line_range",
                "zero_line",
                "line numbers are one-based",
            ));
        }
        if end < start {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidRange,
                "line_range",
                format_range_subject(start, end),
                "line range end precedes start",
            ));
        }
        Ok(Self { start, end })
    }

    /// Returns the first line in the inclusive range.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Returns the last line in the inclusive range.
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Returns true when this range overlaps another range.
    #[must_use]
    pub const fn overlaps(self, other: Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    fn contains(self, line: u32) -> bool {
        self.start <= line && line <= self.end
    }
}

/// Diff hunk shape over original and resulting line coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceDiffHunk {
    old_start: u32,
    old_len: u32,
    new_start: u32,
    new_len: u32,
}

impl ProvenanceDiffHunk {
    /// Builds a validated diff hunk descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when starts are zero, both lengths are zero,
    /// or range arithmetic would overflow.
    pub fn new(
        old_start: u32,
        old_len: u32,
        new_start: u32,
        new_len: u32,
    ) -> Result<Self, ProvenanceError> {
        if old_start < MIN_LINE_NUMBER || new_start < MIN_LINE_NUMBER {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidRange,
                "diff_hunk",
                "zero_start",
                "hunk starts are one-based",
            ));
        }
        if old_len == ZERO_LENGTH && new_len == ZERO_LENGTH {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::InvalidRange,
                "diff_hunk",
                old_start.to_string(),
                "hunk cannot have zero old and new length",
            ));
        }
        validate_hunk_end("old_hunk", old_start, old_len)?;
        validate_hunk_end("new_hunk", new_start, new_len)?;
        Ok(Self {
            old_start,
            old_len,
            new_start,
            new_len,
        })
    }

    /// Returns the original-file start line.
    #[must_use]
    pub const fn old_start(self) -> u32 {
        self.old_start
    }

    /// Returns the original-file line count.
    #[must_use]
    pub const fn old_len(self) -> u32 {
        self.old_len
    }

    /// Returns the resulting-file start line.
    #[must_use]
    pub const fn new_start(self) -> u32 {
        self.new_start
    }

    /// Returns the resulting-file line count.
    #[must_use]
    pub const fn new_len(self) -> u32 {
        self.new_len
    }

    fn old_end_for_order(self) -> Result<u32, ProvenanceError> {
        if self.old_len == ZERO_LENGTH {
            Ok(self.old_start)
        } else {
            self.old_start
                .checked_add(self.old_len - MIN_LINE_NUMBER)
                .ok_or_else(|| overflow_error("old_hunk", self.old_start))
        }
    }

    fn removes_original_line(self, line: u32) -> Result<bool, ProvenanceError> {
        if self.old_len == ZERO_LENGTH {
            return Ok(false);
        }
        let range = ProvenanceLineRange::new(self.old_start, self.old_end_for_order()?)?;
        Ok(range.contains(line))
    }

    fn shifts_original_line(self, line: u32) -> Result<bool, ProvenanceError> {
        if self.old_len == ZERO_LENGTH {
            return Ok(line >= self.old_start);
        }
        Ok(line > self.old_end_for_order()?)
    }

    fn delta(self) -> i64 {
        i64::from(self.new_len) - i64::from(self.old_len)
    }
}

/// Result of transforming attributed line ranges through diff hunks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRangeTransform {
    /// Preserved line ranges in resulting-file coordinates.
    pub preserved_ranges: Collection<ProvenanceLineRange>,
    /// Count of original attributed lines still accepted after transformation.
    pub accepted_line_count: u64,
}
