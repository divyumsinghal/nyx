use std::fmt;

use nun::{NyxError, Result, error::FieldError};

const MAX_SEGMENT_LEN: usize = 128;
const MAX_KEY_LEN: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoryId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HighlightId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaEntity {
    Stories,
    Highlights,
}

impl MediaEntity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stories => "stories",
            Self::Highlights => "highlights",
        }
    }
}

impl StoryId {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_segment("story_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl HighlightId {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_segment("highlight_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ObjectKey {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_object_key(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn entity(&self) -> &str {
        self.0.split('/').nth(1).unwrap_or_default()
    }
}

impl fmt::Display for StoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for HighlightId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for StoryId {
    type Error = NyxError;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for HighlightId {
    type Error = NyxError;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for ObjectKey {
    type Error = NyxError;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

pub fn story_object_key(
    app: &str,
    story_id: &StoryId,
    variant: &str,
    extension: &str,
) -> Result<ObjectKey> {
    object_key(
        app,
        MediaEntity::Stories,
        story_id.as_str(),
        variant,
        extension,
    )
}

pub fn highlight_object_key(
    app: &str,
    highlight_id: &HighlightId,
    variant: &str,
    extension: &str,
) -> Result<ObjectKey> {
    object_key(
        app,
        MediaEntity::Highlights,
        highlight_id.as_str(),
        variant,
        extension,
    )
}

fn object_key(
    app: &str,
    entity: MediaEntity,
    object_id: &str,
    variant: &str,
    extension: &str,
) -> Result<ObjectKey> {
    validate_segment("app", app)?;
    validate_segment("variant", variant)?;
    validate_extension(extension)?;

    let key = format!(
        "{}/{}/{}/{}.{}",
        app,
        entity.as_str(),
        object_id,
        variant,
        extension
    );

    ObjectKey::parse(key)
}

fn validate_object_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(field_error(
            "object_key",
            "required",
            "Object key is required",
        ));
    }

    if key.len() > MAX_KEY_LEN {
        return Err(field_error(
            "object_key",
            "too_long",
            format!("Object key must be at most {MAX_KEY_LEN} characters"),
        ));
    }

    if key.starts_with('/') || key.ends_with('/') || key.contains("//") {
        return Err(field_error(
            "object_key",
            "invalid_format",
            "Object key must not start/end with '/' or contain empty segments",
        ));
    }

    if key.contains("../")
        || key.contains("/../")
        || key.contains("./")
        || key.contains("/./")
        || key.contains('\\')
        || key.contains('\0')
    {
        return Err(field_error(
            "object_key",
            "invalid_format",
            "Object key contains unsafe path traversal markers",
        ));
    }

    let mut segments = key.split('/');
    let app = segments.next();
    let entity = segments.next();
    let object_id = segments.next();
    let file = segments.next();
    let extra = segments.next();

    if app.is_none() || entity.is_none() || object_id.is_none() || file.is_none() || extra.is_some()
    {
        return Err(field_error(
            "object_key",
            "invalid_format",
            "Object key must match '{app}/{entity}/{id}/{variant}.{ext}'",
        ));
    }

    let app = app.unwrap_or_default();
    let entity = entity.unwrap_or_default();
    let object_id = object_id.unwrap_or_default();
    let file = file.unwrap_or_default();

    validate_segment("app", app)?;
    validate_segment("object_id", object_id)?;

    match entity {
        "stories" | "highlights" => {}
        _ => {
            return Err(field_error(
                "entity",
                "invalid_value",
                "Entity must be either 'stories' or 'highlights'",
            ));
        }
    }

    let Some((variant, extension)) = file.rsplit_once('.') else {
        return Err(field_error(
            "object_key",
            "invalid_format",
            "Object filename must include extension as '{variant}.{ext}'",
        ));
    };

    validate_segment("variant", variant)?;
    validate_extension(extension)
}

fn validate_segment(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(field_error(field, "required", "Segment must not be empty"));
    }

    if value.len() > MAX_SEGMENT_LEN {
        return Err(field_error(
            field,
            "too_long",
            format!("Segment must be at most {MAX_SEGMENT_LEN} characters"),
        ));
    }

    if value == "." || value == ".." {
        return Err(field_error(
            field,
            "invalid_format",
            "'.' and '..' are not allowed",
        ));
    }

    if value.contains('/') || value.contains('\\') {
        return Err(field_error(
            field,
            "invalid_format",
            "Path separator characters are not allowed",
        ));
    }

    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(field_error(
            field,
            "invalid_format",
            "Only ASCII letters, digits, '-' and '_' are allowed",
        ));
    }

    Ok(())
}

fn validate_extension(extension: &str) -> Result<()> {
    if extension.is_empty() {
        return Err(field_error(
            "extension",
            "required",
            "File extension is required",
        ));
    }

    if extension.len() > 10 {
        return Err(field_error(
            "extension",
            "too_long",
            "File extension must be at most 10 characters",
        ));
    }

    if !extension
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    {
        return Err(field_error(
            "extension",
            "invalid_format",
            "File extension must be lowercase alphanumeric",
        ));
    }

    Ok(())
}

fn field_error(
    field: &'static str,
    code: &'static str,
    message: impl Into<std::borrow::Cow<'static, str>>,
) -> NyxError {
    NyxError::validation(vec![FieldError::new(field, code, message)])
}
