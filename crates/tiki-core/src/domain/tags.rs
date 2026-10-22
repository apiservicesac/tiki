/// Tags cut across projects (#billing, #urgent). Stored lowercase, without the "#", with
/// dashes instead of spaces, each once.
pub struct Tags;

impl Tags {
    /// The longest tag kept; longer ones are cut.
    const MAX_LENGTH: usize = 32;

    pub fn normalize<S: AsRef<str>>(tags: &[S]) -> Vec<String> {
        let mut normalized: Vec<String> = Vec::new();
        for tag in tags {
            let tag: String = tag
                .as_ref()
                .trim()
                .trim_start_matches('#')
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-")
                .to_lowercase()
                .chars()
                .take(Self::MAX_LENGTH)
                .collect();
            if !tag.is_empty() && !normalized.contains(&tag) {
                normalized.push(tag);
            }
        }
        normalized
    }
}
