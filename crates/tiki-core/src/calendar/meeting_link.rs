/// The link that joins a meeting. A Google Meet link opens with the account the meeting
/// comes from, so someone signed in to several Google accounts joins as the right one.
pub struct MeetingLink;

impl MeetingLink {
    const GOOGLE_MEET: &'static str = "https://meet.google.com/";

    pub fn for_account(join_url: &str, account_email: &str) -> String {
        if join_url.starts_with(Self::GOOGLE_MEET) {
            format!("{join_url}?authuser={account_email}")
        } else {
            join_url.to_owned()
        }
    }
}
