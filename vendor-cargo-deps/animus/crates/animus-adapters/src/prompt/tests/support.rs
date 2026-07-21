use super::super::*;

pub(super) const SAMPLE_FILE: &str = "@src/lib.rs:2-4";
pub(super) const SAMPLE_DIRECTORY: &str = "@docs/";
pub(super) const SAMPLE_DIFF: &str = "@diff:working";
pub(super) const SAMPLE_IMAGE: &str = "@image:assets/logo.png";
pub(super) const SAMPLE_URL: &str = "@https://example.test/docs";
pub(super) const MALFORMED_RANGE: &str = "@src/lib.rs:4-2";
pub(super) const UNSUPPORTED_REFERENCE: &str = "@ftp://example.test/archive";
pub(super) const ESCAPING_REFERENCE: &str = "@../secrets.txt";
pub(super) const FILE_CONTENT: &str = "alpha\nbeta\ngamma\n";
pub(super) const SHORT_CONTENT: &str = "alpha\n";
pub(super) const MAX_VISIBLE_BYTES: u32 = 10;
pub(super) const MAX_VISIBLE_LINES: u32 = 2;
pub(super) const OVERLONG_SAFE_LABEL: &str = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";

pub(super) struct CountingUrlResolver<'a> {
    pub(super) calls: core::cell::Cell<u32>,
    content: &'a str,
}

impl<'a> CountingUrlResolver<'a> {
    pub(super) const fn new(content: &'a str) -> Self {
        Self {
            calls: core::cell::Cell::new(super::super::base::ZERO_COUNT),
            content,
        }
    }
}

impl UrlContextResolver for CountingUrlResolver<'_> {
    fn resolve_url(
        &self,
        reference: &ContextReference,
    ) -> Result<ResolvedContextCandidate, ContextReferenceError> {
        self.calls.set(
            self.calls
                .get()
                .saturating_add(super::super::base::MIN_LINE_NUMBER),
        );
        Ok(ResolvedContextCandidate::new(
            reference.clone(),
            ContextResolverKind::Url,
            self.content,
        ))
    }
}
