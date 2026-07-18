/// OpenCode-style output truncation to protect context windows.
pub fn truncate_output(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        return s.to_string();
    }
    let head = max_chars * 3 / 4;
    let tail = max_chars / 4;
    let head = s.chars().take(head).collect::<String>();
    let tail: String = s
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!(
        "{head}\n\n… truncated {} chars …\n\n{tail}",
        s.len() - max_chars
    )
}

pub const DEFAULT_MAX: usize = 24_000;
