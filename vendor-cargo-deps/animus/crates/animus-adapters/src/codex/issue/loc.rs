/// Borrowed affected-span descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewSpanDraft<'a> {
    /// Source label from the reviewed change set.
    pub source_label: &'a str,
    /// First affected line, one-based.
    pub start_line: u32,
    /// Last affected line, one-based.
    pub end_line: u32,
    /// Optional first affected column, one-based.
    pub start_column: Option<u32>,
    /// Optional last affected column, one-based.
    pub end_column: Option<u32>,
}

/// Validated affected span for a review finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewSpan {
    source_label: super::super::Text,
    start_line: u32,
    end_line: u32,
    start_column: Option<u32>,
    end_column: Option<u32>,
}

impl DiffReviewSpan {
    /// Validates an affected span without opening the source file.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::DiffReviewError`] when labels are malformed, line
    /// numbers are zero, ranges are reversed, or same-line columns are impossible.
    pub fn validate(
        draft: &DiffReviewSpanDraft<'_>,
    ) -> Result<Self, super::super::DiffReviewError> {
        let source_label = super::super::check::required_safe_text(
            super::super::SOURCE_LABEL_FIELD,
            draft.source_label,
        )?;
        let lines = super::super::check::LineRange {
            start_line: draft.start_line,
            end_line: draft.end_line,
        };
        let columns = super::super::check::ColumnRange {
            lines,
            start_column: draft.start_column,
            end_column: draft.end_column,
        };
        super::super::check::span_lines(lines)?;
        super::super::check::span_columns(columns)?;
        Ok(Self {
            source_label,
            start_line: draft.start_line,
            end_line: draft.end_line,
            start_column: draft.start_column,
            end_column: draft.end_column,
        })
    }

    /// Returns the reviewed source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the first affected line.
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start_line
    }

    /// Returns the last affected line.
    #[must_use]
    pub const fn end_line(&self) -> u32 {
        self.end_line
    }

    /// Returns the optional first affected column.
    #[must_use]
    pub const fn start_column(&self) -> Option<u32> {
        self.start_column
    }

    /// Returns the optional last affected column.
    #[must_use]
    pub const fn end_column(&self) -> Option<u32> {
        self.end_column
    }
}
