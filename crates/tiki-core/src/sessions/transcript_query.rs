/// Turns what the user typed into an FTS5 query that cannot break its syntax: each word is
/// quoted (`"word"*`, quotes inside doubled) and matches as a prefix, and every word must
/// appear.
pub struct TranscriptQuery;

impl TranscriptQuery {
    /// The query, or `None` when the text has no words.
    pub fn from_text(text: &str) -> Option<String> {
        let words: Vec<String> = text
            .split_whitespace()
            .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
            .collect();
        (!words.is_empty()).then(|| words.join(" "))
    }
}
