use tiki_core::i18n::Translator;

use crate::desktop_config::DesktopConfig;

/// The page the browser shows when it comes back from the provider: Tiki's face, whether
/// the calendar got connected and one line on what to do next. It tries to close itself
/// after a moment, since the user goes back to Tiki anyway.
pub struct SignInPages {
    connected: String,
    failed: String,
}

impl SignInPages {
    const TEMPLATE: &'static str = include_str!("sign_in_page.html");
    /// Tiki's calm amber and its worried red, as in the stylesheet's mood tokens.
    const CONNECTED_EYES: &'static str = "#f6c768";
    const FAILED_EYES: &'static str = "#f4505e";

    pub fn new(translator: &Translator) -> Self {
        Self {
            connected: Self::render(
                translator,
                "calendar.sign_in.connected",
                "calendar.sign_in.connected_hint",
                Self::CONNECTED_EYES,
            ),
            failed: Self::render(
                translator,
                "calendar.sign_in.failed",
                "calendar.sign_in.failed_hint",
                Self::FAILED_EYES,
            ),
        }
    }

    /// The page for the provider's answer: connected, or not.
    pub fn for_answer(&self, connected: bool) -> &str {
        if connected {
            &self.connected
        } else {
            &self.failed
        }
    }

    fn render(translator: &Translator, title_key: &str, hint_key: &str, eyes: &str) -> String {
        Self::TEMPLATE
            .replace("{{lang}}", translator.locale().code())
            .replace("{{eye}}", eyes)
            .replace("{{title}}", &Self::escape(&translator.text(title_key)))
            .replace("{{hint}}", &Self::escape(&translator.text(hint_key)))
            .replace(
                "{{close_after_ms}}",
                &DesktopConfig::SIGN_IN_PAGE_CLOSE_AFTER
                    .as_millis()
                    .to_string(),
            )
    }

    fn escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}
