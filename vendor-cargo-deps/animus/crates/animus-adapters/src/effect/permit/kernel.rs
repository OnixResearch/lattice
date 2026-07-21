#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

use alloc::string::ToString;

const GLOB_MATCH_LABEL: &str = "glob";
const REGEX_MATCH_LABEL: &str = "regex";
const RESTRICTIVE_MODE_LABEL: &str = "restrictive";
const READ_ONLY_MODE_LABEL: &str = "read-only";
const GUARDED_MODE_LABEL: &str = "guarded";
const STANDARD_MODE_LABEL: &str = "standard";
const YOLO_MODE_LABEL: &str = "yolo";
const COACH_REPEAT_LABEL: &str = "coach";
const ASK_REPEAT_LABEL: &str = "ask";
const DENY_REPEAT_LABEL: &str = "deny";
const IGNORE_REPEAT_LABEL: &str = "ignore";
const GLOB_WILDCARD: char = '*';
const REGEX_CLASS_START: char = '[';
const REGEX_CLASS_END: char = ']';
const REGEX_ESCAPE: char = '\\';
const MIN_REPEAT_THRESHOLD: u32 = 1;

const _: () = assert!(MIN_REPEAT_THRESHOLD > 0);

include!("kernel/mode.rs");
include!("kernel/patterntypes.rs");
include!("kernel/patternmatch.rs");
include!("kernel/session.rs");
include!("kernel/repeated.rs");
include!("kernel/receipt.rs");
include!("kernel/parse.rs");
