use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use tiki_core::calendar::Meeting;
use tiki_core::{TikiError, TikiResult};
use url::Url;

use super::provider_answer::ProviderAnswer;

/// Reads the meetings of a Microsoft 365 or Outlook.com calendar (Microsoft Graph).
pub struct MicrosoftCalendar<'http> {
    http: &'http reqwest::Client,
}

#[derive(Deserialize)]
struct EventPage {
    value: Vec<GraphEvent>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphEvent {
    id: String,
    subject: Option<String>,
    start: GraphTime,
    end: GraphTime,
    is_all_day: bool,
    is_cancelled: bool,
    location: Option<GraphLocation>,
    online_meeting: Option<GraphOnlineMeeting>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphTime {
    /// UTC, because Tiki asks for it with the `Prefer` header.
    date_time: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphLocation {
    display_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphOnlineMeeting {
    join_url: Option<String>,
}

impl<'http> MicrosoftCalendar<'http> {
    const CALENDAR_VIEW: &'static str = "https://graph.microsoft.com/v1.0/me/calendarView";

    pub fn new(http: &'http reqwest::Client) -> Self {
        Self { http }
    }

    /// Every meeting between `from` and `to` (local time), cancelled ones left out.
    pub async fn meetings(
        &self,
        access_token: &str,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> TikiResult<Vec<Meeting>> {
        let mut url = Url::parse(Self::CALENDAR_VIEW).map_err(|error| Self::error(&error))?;
        url.query_pairs_mut()
            .append_pair("startDateTime", &Self::utc(from))
            .append_pair("endDateTime", &Self::utc(to))
            .append_pair(
                "$select",
                "id,subject,start,end,isAllDay,isCancelled,location,onlineMeeting",
            )
            .append_pair("$top", "200");
        let mut next = Some(url.to_string());
        let mut meetings = Vec::new();
        while let Some(page_url) = next {
            let response = self
                .http
                .get(&page_url)
                .bearer_auth(access_token)
                .header("Prefer", "outlook.timezone=\"UTC\"")
                .send()
                .await
                .map_err(|error| Self::error(&error))?;
            let body = ProviderAnswer::body(response)
                .await
                .map_err(|error| Self::error(&error))?;
            let page: EventPage =
                serde_json::from_str(&body).map_err(|error| Self::error(&error))?;
            meetings.extend(
                page.value
                    .into_iter()
                    .filter(|event| !event.is_cancelled)
                    .filter_map(Self::meeting),
            );
            next = page.next_link;
        }
        Ok(meetings)
    }

    fn meeting(event: GraphEvent) -> Option<Meeting> {
        Some(Meeting {
            external_id: event.id,
            title: event.subject.unwrap_or_default(),
            starts_at: Self::local(&event.start.date_time)?,
            ends_at: Self::local(&event.end.date_time)?,
            all_day: event.is_all_day,
            location: event
                .location
                .and_then(|location| location.display_name)
                .unwrap_or_default(),
            join_url: event.online_meeting.and_then(|meeting| meeting.join_url),
        })
    }

    /// Graph writes UTC times without an offset, sometimes with fractions of a second.
    fn local(utc: &str) -> Option<NaiveDateTime> {
        let naive =
            NaiveDateTime::parse_from_str(utc.split('.').next()?, "%Y-%m-%dT%H:%M:%S").ok()?;
        Some(
            Utc.from_utc_datetime(&naive)
                .with_timezone(&Local)
                .naive_local(),
        )
    }

    fn utc(local: NaiveDateTime) -> String {
        Local
            .from_local_datetime(&local)
            .earliest()
            .map_or_else(|| local.and_utc(), |time| time.with_timezone(&Utc))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    }

    fn error(error: &impl std::fmt::Display) -> TikiError {
        TikiError::Calendar(format!("Microsoft calendar: {error}"))
    }
}

/// Parses an RFC 3339 time into local time; shared with the Google reader.
pub fn local_from_rfc3339(text: &str) -> Option<NaiveDateTime> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|time| time.with_timezone(&Local).naive_local())
}
