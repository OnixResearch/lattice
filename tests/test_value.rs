//! Tests for engine::value — ContentHash, Value, TypeTag.

mod common;

use lattice::engine::value::ContentHash;
use lattice::engine::value::ImageMetadata;
use lattice::engine::value::ImageValue;
use lattice::engine::value::TypeTag;
use lattice::engine::value::Value;
use lattice::engine::value::ValueData;

const PNG_MEDIA_TYPE: &str = "image/png";
const JPEG_MEDIA_TYPE: &str = "image/jpeg";
const IMAGE_BYTES_A: &[u8] = b"image-a";
const IMAGE_BYTES_B: &[u8] = b"image-b";
use pretty_assertions::assert_eq;

// ---------------------------------------------------------------------------
// ContentHash
// ---------------------------------------------------------------------------

#[test]
fn hash_deterministic() {
    let a = ContentHash::from_bytes(b"hello");
    let b = ContentHash::from_bytes(b"hello");
    assert_eq!(a, b, "same input must produce the same hash");
}

#[test]
fn hash_differs_for_different_input() {
    let a = ContentHash::from_bytes(b"hello");
    let b = ContentHash::from_bytes(b"world");
    assert_ne!(a, b);
}

#[test]
fn hash_empty_input() {
    let h = ContentHash::from_bytes(b"");
    // blake3("") is a known constant — just check it's not zero.
    assert_ne!(h.0, [0u8; 32]);
}

#[test]
fn hash_hex_round_trip() {
    let h = ContentHash::from_bytes(b"round trip test");
    let hex = h.to_hex();
    assert_eq!(hex.len(), 64, "blake3 hex is 64 chars");
    // Parse back through blake3::Hash.
    let reparsed = blake3::Hash::from_hex(&hex).unwrap();
    assert_eq!(*reparsed.as_bytes(), h.0);
}

#[test]
fn hash_combine_order_matters() {
    let a = ContentHash::from_bytes(b"a");
    let b = ContentHash::from_bytes(b"b");
    let ab = ContentHash::combine(&[a, b]);
    let ba = ContentHash::combine(&[b, a]);
    assert_ne!(ab, ba, "combine is order-sensitive");
}

#[test]
fn hash_combine_empty() {
    let empty = ContentHash::combine(&[]);
    // Combining zero hashes should still produce a valid (deterministic) hash.
    assert_ne!(empty.0, [0u8; 32]);
}

#[test]
fn hash_serde_round_trip() {
    let h = ContentHash::from_bytes(b"serde test");
    let json = serde_json::to_string(&h).unwrap();
    let back: ContentHash = serde_json::from_str(&json).unwrap();
    assert_eq!(h, back);
}

#[test]
fn hash_debug_shows_prefix() {
    let h = ContentHash::from_bytes(b"debug fmt");
    let dbg = format!("{:?}", h);
    assert!(dbg.contains('…'), "debug format should contain ellipsis: {dbg}");
    // Should be a short prefix, not the full 64-char hex.
    assert!(dbg.len() < 64, "debug format should be truncated: {dbg}");
}

// ---------------------------------------------------------------------------
// Value construction and type_tag
// ---------------------------------------------------------------------------

#[test]
fn value_from_string() {
    let v = Value::from_string("hello".into());
    assert_eq!(v.type_tag(), TypeTag::String);
    assert_eq!(v.as_display_string(), "hello");
    assert!(!v.is_error());
}

#[test]
fn value_from_bytes() {
    let v = Value::from_bytes(vec![0xDE, 0xAD]);
    assert_eq!(v.type_tag(), TypeTag::Bytes);
    assert!(!v.is_error());
}

#[test]
fn value_from_json() {
    let j = serde_json::json!({"key": "val"});
    let v = Value::from_json(j.clone());
    assert_eq!(v.type_tag(), TypeTag::Json);
    let display = v.as_display_string();
    assert!(display.contains("key"));
}

// r[verify n8nclone.workflow_execution.image_values.model]
#[test]
fn value_from_image_is_typed_and_display_safe() {
    let v = Value::from_image(ImageValue::new(IMAGE_BYTES_A.to_vec(), PNG_MEDIA_TYPE.to_string(), ImageMetadata {
        width: Some(32),
        height: Some(16),
        source_label: Some("fixture.png".to_string()),
    }));

    assert_eq!(v.type_tag(), TypeTag::Image);
    let display = v.as_display_string();
    assert!(display.contains(PNG_MEDIA_TYPE));
    assert!(display.contains("32x16"));
    assert!(!display.contains("image-a"), "image display must not dump raw bytes");
}

// r[verify n8nclone.workflow_execution.image_values.model]
#[test]
fn image_hash_includes_bytes_and_media_type() {
    let png_a = Value::from_image(ImageValue::new(
        IMAGE_BYTES_A.to_vec(),
        PNG_MEDIA_TYPE.to_string(),
        ImageMetadata::default(),
    ));
    let png_a_again = Value::from_image(ImageValue::new(
        IMAGE_BYTES_A.to_vec(),
        PNG_MEDIA_TYPE.to_string(),
        ImageMetadata::default(),
    ));
    let png_b = Value::from_image(ImageValue::new(
        IMAGE_BYTES_B.to_vec(),
        PNG_MEDIA_TYPE.to_string(),
        ImageMetadata::default(),
    ));
    let jpeg_a = Value::from_image(ImageValue::new(
        IMAGE_BYTES_A.to_vec(),
        JPEG_MEDIA_TYPE.to_string(),
        ImageMetadata::default(),
    ));

    assert_eq!(png_a.hash, png_a_again.hash);
    assert_ne!(png_a.hash, png_b.hash);
    assert_ne!(png_a.hash, jpeg_a.hash);
}

#[test]
fn value_from_exit_code() {
    let v = Value::from_exit_code(42);
    assert_eq!(v.type_tag(), TypeTag::ExitCode);
    assert!(v.as_display_string().contains("42"));
}

#[test]
fn value_error() {
    let v = Value::error("boom".into(), "stack trace".into());
    assert!(v.is_error());
    // Error values report TypeTag::String.
    assert_eq!(v.type_tag(), TypeTag::String);
    let display = v.as_display_string();
    assert!(display.contains("boom"));
    assert!(display.contains("stack trace"));
}

#[test]
fn value_hash_is_content_based() {
    let a = Value::from_string("same".into());
    let b = Value::from_string("same".into());
    assert_eq!(a.hash, b.hash, "identical content must produce identical hash");

    let c = Value::from_string("different".into());
    assert_ne!(a.hash, c.hash);
}

#[test]
fn value_serde_round_trip_string() {
    let v = Value::from_string("round trip".into());
    let json = serde_json::to_string(&v).unwrap();
    let back: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v.hash, back.hash);
    assert_eq!(v.as_display_string(), back.as_display_string());
}

#[test]
fn value_serde_round_trip_json() {
    let v = Value::from_json(serde_json::json!({"nested": [1,2,3]}));
    let json = serde_json::to_string(&v).unwrap();
    let back: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v.hash, back.hash);
}

#[test]
fn value_serde_round_trip_exit_code() {
    let v = Value::from_exit_code(-1);
    let json = serde_json::to_string(&v).unwrap();
    let back: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v.hash, back.hash);
    match back.data {
        ValueData::ExitCode(c) => assert_eq!(c, -1),
        _ => panic!("expected ExitCode variant"),
    }
}

// ---------------------------------------------------------------------------
// TypeTag compatibility
// ---------------------------------------------------------------------------

#[test]
fn type_tag_same_is_compatible() {
    assert!(TypeTag::String.compatible_with(&TypeTag::String));
    assert!(TypeTag::Bytes.compatible_with(&TypeTag::Bytes));
    assert!(TypeTag::Json.compatible_with(&TypeTag::Json));
    assert!(TypeTag::Image.compatible_with(&TypeTag::Image));
    assert!(TypeTag::ExitCode.compatible_with(&TypeTag::ExitCode));
}

#[test]
fn type_tag_any_accepts_everything() {
    assert!(TypeTag::String.compatible_with(&TypeTag::Any));
    assert!(TypeTag::Bytes.compatible_with(&TypeTag::Any));
    assert!(TypeTag::Json.compatible_with(&TypeTag::Any));
    assert!(TypeTag::Image.compatible_with(&TypeTag::Any));
    assert!(TypeTag::ExitCode.compatible_with(&TypeTag::Any));
    assert!(TypeTag::Any.compatible_with(&TypeTag::Any));
}

#[test]
fn type_tag_mismatches() {
    assert!(!TypeTag::String.compatible_with(&TypeTag::Bytes));
    assert!(!TypeTag::ExitCode.compatible_with(&TypeTag::Json));
    assert!(!TypeTag::Bytes.compatible_with(&TypeTag::String));
    assert!(!TypeTag::Image.compatible_with(&TypeTag::String));
}

#[test]
fn type_tag_as_str_round_trip() {
    for tag in [
        TypeTag::String,
        TypeTag::Bytes,
        TypeTag::Json,
        TypeTag::Image,
        TypeTag::ExitCode,
        TypeTag::Any,
    ] {
        let s = tag.as_str();
        assert!(!s.is_empty());
        // Display should match as_str.
        assert_eq!(format!("{}", tag), s);
    }
}
