// File validation helpers
pub fn is_allowed_extension(filename: &str) -> bool {
    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
    {
        let allowed = [
            "pdf", "docx", "ppt", "pptx", "zip", "jpg", "jpeg", "png", "gif",
        ];
        return allowed.contains(&ext.to_lowercase().as_str());
    }
    false
}
