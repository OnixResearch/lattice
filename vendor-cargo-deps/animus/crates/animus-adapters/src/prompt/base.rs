pub(in crate::prompt) const REFERENCE_SIGIL: char = '@';
pub(in crate::prompt) const LOCAL_RANGE_SEPARATOR: char = ':';
pub(in crate::prompt) const RANGE_SPAN_SEPARATOR: char = '-';
pub(in crate::prompt) const PATH_SEPARATOR: char = '/';
pub(in crate::prompt) const WINDOWS_PATH_SEPARATOR: char = '\\';
pub(in crate::prompt) const CURRENT_DIR_PREFIX: &str = "./";
pub(in crate::prompt) const PARENT_DIR_SEGMENT: &str = "..";
pub(in crate::prompt) const HOME_DIR_PREFIX: &str = "~";
pub(in crate::prompt) const HTTP_PREFIX: &str = "http://";
pub(in crate::prompt) const HTTPS_PREFIX: &str = "https://";
pub(in crate::prompt) const FTP_PREFIX: &str = "ftp://";
pub(in crate::prompt) const DIFF_PREFIX: &str = "diff:";
pub(in crate::prompt) const IMAGE_PREFIX: &str = "image:";
pub(in crate::prompt) const URL_LABEL_PREFIX: &str = "url:";
pub(in crate::prompt) const EMPTY_TARGET_REASON: &str = "context reference target must not be empty";
pub(in crate::prompt) const PATH_ESCAPE_REASON: &str = "context reference target must stay relative";
pub(in crate::prompt) const UNSUPPORTED_KIND_REASON: &str = "context reference kind is not supported";
pub(in crate::prompt) const MALFORMED_RANGE_REASON: &str = "context reference line range is malformed";
pub(in crate::prompt) const URL_CREDENTIAL_REASON: &str = "url reference authority must not contain credentials";
pub(in crate::prompt) const BYTE_COUNT_LABEL: &str = "byte_count";
pub(in crate::prompt) const LINE_COUNT_LABEL: &str = "line_count";
pub(in crate::prompt) const VISIBLE_BYTE_COUNT_LABEL: &str = "visible_byte_count";
pub(in crate::prompt) const VISIBLE_LINE_COUNT_LABEL: &str = "visible_line_count";
pub(in crate::prompt) const CONTEXT_BOUNDS_FIELD: &str = "context_reference_bounds";
pub(in crate::prompt) const KIND_FILE_BIT: u8 = 1;
pub(in crate::prompt) const KIND_DIRECTORY_BIT: u8 = KIND_FILE_BIT << 1;
pub(in crate::prompt) const KIND_DIFF_BIT: u8 = KIND_DIRECTORY_BIT << 1;
pub(in crate::prompt) const KIND_IMAGE_BIT: u8 = KIND_DIFF_BIT << 1;
pub(in crate::prompt) const KIND_URL_BIT: u8 = KIND_IMAGE_BIT << 1;
pub(in crate::prompt) const ALL_LOCAL_KINDS: u8 = KIND_FILE_BIT | KIND_DIRECTORY_BIT | KIND_DIFF_BIT | KIND_IMAGE_BIT;
pub(in crate::prompt) const ALL_CONTEXT_KINDS: u8 = ALL_LOCAL_KINDS | KIND_URL_BIT;
pub(in crate::prompt) const MIN_LINE_NUMBER: u32 = 1;
pub(in crate::prompt) const ZERO_COUNT: u32 = 0;
pub(in crate::prompt) const ZERO_COUNT_USIZE: usize = 0;
pub(in crate::prompt) const MIN_LINE_COUNT_USIZE: usize = 1;
pub(in crate::prompt) const ZERO_ALLOWED_KINDS: u8 = 0;
pub(in crate::prompt) const EMPTY_HASH_INPUT: &str = "";

pub(in crate::prompt) fn usize_to_u32(value: usize) -> Result<u32, super::data::ContextReferenceError> {
    u32::try_from(value).map_err(|_error| super::data::ContextReferenceError::CountOverflow)
}

pub(in crate::prompt) fn hash_text(text: &str) -> crate::ArtifactHash {
    let hash = blake3::hash(text.as_bytes());
    crate::ArtifactHash::from_bytes(hash.as_bytes())
}
