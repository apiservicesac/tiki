use serde::Serialize;

/// The service a meeting's video call runs on, told by the host of its join link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeetingPlatform {
    GoogleMeet,
    Teams,
    Zoom,
    Other,
}

impl MeetingPlatform {
    /// Each platform and the end of the hosts its links use.
    const HOSTS: [(&'static str, MeetingPlatform); 4] = [
        ("meet.google.com", MeetingPlatform::GoogleMeet),
        ("teams.microsoft.com", MeetingPlatform::Teams),
        ("teams.live.com", MeetingPlatform::Teams),
        ("zoom.us", MeetingPlatform::Zoom),
    ];

    /// The platform of a join link such as `https://meet.google.com/abc-defg-hij`.
    pub fn of(join_url: &str) -> Self {
        let host = join_url.split('/').nth(2).unwrap_or_default();
        Self::HOSTS
            .into_iter()
            .find(|(suffix, _)| host.ends_with(suffix))
            .map_or(MeetingPlatform::Other, |(_, platform)| platform)
    }

    /// Stable key, used in translations.
    pub fn key(self) -> &'static str {
        match self {
            MeetingPlatform::GoogleMeet => "google_meet",
            MeetingPlatform::Teams => "teams",
            MeetingPlatform::Zoom => "zoom",
            MeetingPlatform::Other => "other",
        }
    }
}
