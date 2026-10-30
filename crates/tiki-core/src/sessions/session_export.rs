use super::{SessionDetails, Transcript};
use crate::i18n::Translator;

/// A session written out to share or keep elsewhere: the transcript as subtitles (SRT) and
/// as plain text, and the notes as Markdown, in the user's language.
pub struct SessionExport<'translator> {
    translator: &'translator Translator,
}

impl<'translator> SessionExport<'translator> {
    /// Characters Windows does not allow in a folder name.
    const FORBIDDEN_IN_NAMES: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    const NAME_REPLACEMENT: char = '-';
    const MS_PER_SECOND: i64 = 1_000;
    const SECONDS_PER_MINUTE: i64 = 60;
    const SECONDS_PER_HOUR: i64 = 3_600;

    pub fn new(translator: &'translator Translator) -> Self {
        Self { translator }
    }

    /// "Tiki · 2026-10-21 · Revisión con Atlas", safe as a Windows folder name.
    pub fn folder_name(&self, details: &SessionDetails) -> String {
        let name = self.translator.format(
            "session.export.folder",
            &[
                (
                    "date",
                    &details
                        .summary
                        .started_at
                        .date()
                        .format("%Y-%m-%d")
                        .to_string(),
                ),
                ("title", &details.summary.title),
            ],
        );
        name.chars()
            .map(|character| {
                if Self::FORBIDDEN_IN_NAMES.contains(&character) || character.is_control() {
                    Self::NAME_REPLACEMENT
                } else {
                    character
                }
            })
            .collect::<String>()
            .trim()
            .to_owned()
    }

    /// SRT subtitles: each line with its time and who said it.
    pub fn subtitles(&self, transcript: &Transcript) -> String {
        transcript
            .segments()
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                format!(
                    "{}\n{} --> {}\n{}\n",
                    index + 1,
                    Self::subtitle_time(segment.start_ms),
                    Self::subtitle_time(segment.end_ms),
                    self.translator.format(
                        "session.export.line",
                        &[
                            ("speaker", &self.speaker(segment.speaker)),
                            ("text", &segment.text)
                        ],
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// "[00:01:05] Tú: Mañana envío el informe", one line each.
    pub fn text(&self, transcript: &Transcript) -> String {
        transcript
            .segments()
            .iter()
            .map(|segment| {
                self.translator.format(
                    "session.export.text_line",
                    &[
                        ("time", &Self::clock(segment.start_ms)),
                        ("speaker", &self.speaker(segment.speaker)),
                        ("text", &segment.text),
                    ],
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The notes as Markdown: the title and start, the summary, key points, decisions and
    /// the proposed tasks (checked once applied). `None` without notes.
    pub fn notes(&self, details: &SessionDetails) -> Option<String> {
        let notes = details.notes.as_ref()?;
        let mut lines = vec![
            format!("# {}", details.summary.title),
            String::new(),
            details.summary.started_label.clone(),
            String::new(),
            format!("## {}", self.translator.text("session.export.summary")),
            String::new(),
            notes.summary.clone(),
        ];
        for (title_key, items) in [
            ("session.notes.key_points", &notes.key_points),
            ("session.notes.decisions", &notes.decisions),
        ] {
            if !items.is_empty() {
                lines.extend([
                    String::new(),
                    format!("## {}", self.translator.text(title_key)),
                    String::new(),
                ]);
                lines.extend(items.iter().map(|item| format!("- {item}")));
            }
        }
        if !details.action_items.is_empty() {
            lines.extend([
                String::new(),
                format!("## {}", self.translator.text("session.notes.tasks")),
                String::new(),
            ]);
            lines.extend(details.action_items.iter().map(|item| {
                let mark = if item.task_id.is_some() { "x" } else { " " };
                format!("- [{mark}] {}", item.title)
            }));
        }
        lines.push(String::new());
        Some(lines.join("\n"))
    }

    fn speaker(&self, speaker: super::Speaker) -> String {
        self.translator
            .text(&format!("session.speaker.{}", speaker.key()))
    }

    /// "01:02:05,250", as SRT writes times.
    fn subtitle_time(ms: i64) -> String {
        format!("{},{:03}", Self::clock(ms), ms % Self::MS_PER_SECOND)
    }

    /// "01:02:05".
    fn clock(ms: i64) -> String {
        let seconds = ms / Self::MS_PER_SECOND;
        format!(
            "{:02}:{:02}:{:02}",
            seconds / Self::SECONDS_PER_HOUR,
            seconds % Self::SECONDS_PER_HOUR / Self::SECONDS_PER_MINUTE,
            seconds % Self::SECONDS_PER_MINUTE
        )
    }
}
