/// Format seconds as "M:SS".
pub fn format_duration(secs: u64) -> String {
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{minutes}:{seconds:02}")
}

/// Format seconds as "H:MM:SS" when >= 1 hour, otherwise "M:SS".
pub fn format_duration_long(secs: u64) -> String {
    if secs >= 3600 {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format_duration(secs)
    }
}

/// Format byte count as human-readable size.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Truncate text to max_len characters, appending "..." if truncated.
pub fn truncate(text: &str, max_len: usize) -> String {
    if max_len < 4 {
        return text.chars().take(max_len).collect();
    }
    let char_count = text.chars().count();
    if char_count <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len - 3).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0:00");
        assert_eq!(format_duration(5), "0:05");
        assert_eq!(format_duration(65), "1:05");
        assert_eq!(format_duration(3599), "59:59");
    }

    #[test]
    fn test_format_duration_long() {
        assert_eq!(format_duration_long(65), "1:05");
        assert_eq!(format_duration_long(3600), "1:00:00");
        assert_eq!(format_duration_long(3661), "1:01:01");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("ab", 2), "ab");
    }
}
