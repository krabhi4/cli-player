"""Utility functions for the CLI Music Player."""


def format_duration(seconds: int) -> str:
    """Format seconds as M:SS or H:MM:SS."""
    if seconds < 0:
        seconds = 0
    hours = seconds // 3600
    minutes = (seconds % 3600) // 60
    secs = seconds % 60
    if hours > 0:
        return f"{hours}:{minutes:02d}:{secs:02d}"
    return f"{minutes}:{secs:02d}"


def format_duration_long(seconds: int) -> str:
    """Format seconds as human-readable (e.g., '3h 24m')."""
    if seconds < 60:
        return f"{seconds}s"
    hours = seconds // 3600
    minutes = (seconds % 3600) // 60
    if hours > 0:
        return f"{hours}h {minutes}m"
    return f"{minutes}m"


def format_size(bytes_val: int) -> str:
    """Format bytes as human-readable size."""
    for unit in ["B", "KB", "MB", "GB"]:
        if bytes_val < 1024:
            return f"{bytes_val:.1f} {unit}"
        bytes_val /= 1024
    return f"{bytes_val:.1f} TB"


def truncate(text: str, max_len: int, suffix: str = "…") -> str:
    """Truncate text to max_len characters."""
    if len(text) <= max_len:
        return text
    return text[: max_len - len(suffix)] + suffix


def progress_bar(current: float, total: float, width: int = 30) -> str:
    """Create a text progress bar."""
    if total <= 0:
        return "░" * width
    ratio = min(1.0, max(0.0, current / total))
    filled = int(width * ratio)
    return "█" * filled + "░" * (width - filled)
