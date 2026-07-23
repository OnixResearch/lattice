//! Image input operation planning and image I/O interface.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::Deserialize;
use serde::Serialize;

use crate::engine::value::ImageMetadata;
use crate::engine::value::ImageValue;
use crate::file_node;

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_START_MARKER: &[u8] = b"\xff\xd8";
const JPEG_END_MARKER: &[u8] = b"\xff\xd9";
const MEDIA_TYPE_PNG: &str = "image/png";
const MEDIA_TYPE_JPEG: &str = "image/jpeg";
const BYTES_PER_KIB: usize = 1024;
const KIB_PER_MIB: usize = 1024;
const DEFAULT_IMAGE_BYTE_BUDGET_MIB: usize = 10;
const IMAGE_IO_NOT_CONFIGURED: &str = "no image I/O configured";
const PNG_EXTENSION: &str = "png";
const JPEG_EXTENSION: &str = "jpeg";
const JPEG_SHORT_EXTENSION: &str = "jpg";
const SUPPORTED_DECLARED_IMAGE_EXTENSIONS: &[&str] = &[PNG_EXTENSION, JPEG_EXTENSION, JPEG_SHORT_EXTENSION];
pub const DEFAULT_MAX_IMAGE_BYTES: usize = DEFAULT_IMAGE_BYTE_BUDGET_MIB * KIB_PER_MIB * BYTES_PER_KIB;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageInputPlan {
    pub path: String,
    pub allowed_roots: Vec<String>,
    pub max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    UnsafePath(String),
    MissingImage(String),
    PermissionDenied(String),
    UnsupportedFormat(String),
    OversizedImage {
        path: String,
        bytes: usize,
        max_bytes: usize,
    },
    Io(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafePath(path) => write!(f, "unsafe image path: {path}"),
            Self::MissingImage(path) => write!(f, "missing image: {path}"),
            Self::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            Self::UnsupportedFormat(path) => write!(f, "unsupported image format: {path}"),
            Self::OversizedImage { path, bytes, max_bytes } => {
                write!(f, "image is too large: {path} has {bytes} bytes, max {max_bytes}")
            }
            Self::Io(message) => write!(f, "image I/O error: {message}"),
        }
    }
}

impl std::error::Error for ImageError {}

pub trait ImageIo: Send + Sync {
    fn load(&self, plan: &ImageInputPlan) -> Result<ImageValue, ImageError>;
}

#[derive(Debug, Default)]
pub struct NoopImageIo;

impl ImageIo for NoopImageIo {
    fn load(&self, _plan: &ImageInputPlan) -> Result<ImageValue, ImageError> {
        Err(ImageError::Io(IMAGE_IO_NOT_CONFIGURED.to_string()))
    }
}

#[derive(Debug, Default)]
pub struct LocalImageIo;

impl ImageIo for LocalImageIo {
    fn load(&self, plan: &ImageInputPlan) -> Result<ImageValue, ImageError> {
        load_local_image(plan)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryImageIo {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    denied_paths: Arc<Mutex<Vec<String>>>,
}

impl MemoryImageIo {
    pub fn new(files: HashMap<String, Vec<u8>>) -> Self {
        Self {
            files: Arc::new(Mutex::new(files)),
            denied_paths: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn deny_path(&self, path: impl Into<String>) {
        self.denied_paths.lock().push(path.into());
    }
}

impl ImageIo for MemoryImageIo {
    fn load(&self, plan: &ImageInputPlan) -> Result<ImageValue, ImageError> {
        validate_image_path_allowed(&plan.path, &plan.allowed_roots)?;
        if self.denied_paths.lock().contains(&plan.path) {
            return Err(ImageError::PermissionDenied(plan.path.clone()));
        }

        let bytes = self
            .files
            .lock()
            .get(&plan.path)
            .cloned()
            .ok_or_else(|| ImageError::MissingImage(plan.path.clone()))?;
        image_value_from_loaded_bytes(&plan.path, bytes, plan.max_bytes)
    }
}

// r[impl n8nclone.workflow_execution.image_input_nodes.load]
pub fn image_input_plan(path: &str, allowed_roots: &[String], max_bytes: usize) -> ImageInputPlan {
    ImageInputPlan {
        path: path.to_string(),
        allowed_roots: allowed_roots.to_vec(),
        max_bytes,
    }
}

// r[impl n8nclone.workflow_execution.image_input_nodes.load]
pub fn validate_image_path_allowed(path: &str, allowed_roots: &[String]) -> Result<(), ImageError> {
    file_node::validate_path_allowed(path, allowed_roots).map_err(|_| ImageError::UnsafePath(path.to_string()))
}

pub fn validate_declared_format(path: &str) -> Result<(), ImageError> {
    let Some(extension) = std::path::Path::new(path).extension().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    if SUPPORTED_DECLARED_IMAGE_EXTENSIONS
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
    {
        Ok(())
    } else {
        Err(ImageError::UnsupportedFormat(path.to_string()))
    }
}

pub fn validate_image_size(path: &str, bytes: usize, max_bytes: usize) -> Result<(), ImageError> {
    if bytes <= max_bytes {
        Ok(())
    } else {
        Err(ImageError::OversizedImage {
            path: path.to_string(),
            bytes,
            max_bytes,
        })
    }
}

// r[impl n8nclone.workflow_execution.image_input_nodes.production_io.local]
pub fn image_value_from_loaded_bytes(path: &str, bytes: Vec<u8>, max_bytes: usize) -> Result<ImageValue, ImageError> {
    validate_image_size(path, bytes.len(), max_bytes)?;
    let media_type = sniff_image_media_type(&bytes).ok_or_else(|| ImageError::UnsupportedFormat(path.to_string()))?;
    Ok(ImageValue::new(bytes, media_type.to_string(), ImageMetadata {
        source_label: Some(path.to_string()),
        ..ImageMetadata::default()
    }))
}

// r[impl n8nclone.workflow_execution.image_input_nodes.production_io.local]
// r[impl n8nclone.workflow_execution.image_input_nodes.capability_read]
pub fn load_local_image(plan: &ImageInputPlan) -> Result<ImageValue, ImageError> {
    let rooted_path = crate::capability::select_rooted_path(&plan.path, &plan.allowed_roots)
        .ok_or_else(|| ImageError::UnsafePath(plan.path.clone()))?;
    let file =
        crate::capability::open_rooted_read(&rooted_path).map_err(|error| image_error_from_io(&plan.path, error))?;
    let metadata = file.metadata().map_err(|error| image_error_from_io(&plan.path, error))?;
    let metadata_bytes = usize::try_from(metadata.len()).map_err(|_| {
        ImageError::Io(format!(
            "image metadata length does not fit this platform: {} has {} bytes",
            plan.path,
            metadata.len()
        ))
    })?;
    validate_image_size(&plan.path, metadata_bytes, plan.max_bytes)?;
    let bytes = crate::capability::read_bounded(file, plan.max_bytes)
        .map_err(|error| image_error_from_io(&plan.path, error))?;
    image_value_from_loaded_bytes(&plan.path, bytes, plan.max_bytes)
}

fn image_error_from_io(path: &str, error: std::io::Error) -> ImageError {
    match error.kind() {
        ErrorKind::NotFound => ImageError::MissingImage(path.to_string()),
        ErrorKind::PermissionDenied => ImageError::PermissionDenied(path.to_string()),
        _ => ImageError::Io(format!("{path}: {error}")),
    }
}

pub fn sniff_image_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(PNG_SIGNATURE) {
        return Some(MEDIA_TYPE_PNG);
    }
    if bytes.starts_with(JPEG_START_MARKER) && bytes.ends_with(JPEG_END_MARKER) {
        return Some(MEDIA_TYPE_JPEG);
    }
    None
}
