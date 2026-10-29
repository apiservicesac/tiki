use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use tiki_core::calendar::Meeting;
use tiki_core::{TikiError, TikiResult};
use url::Url;

use super::microsoft_calendar::local_from_rfc3339;
use super::provider_answer::ProviderAnswer;

/// Reads the meetings of the calendars a Google account shows (Google Calendar API).
pub struct GoogleCalendar<'http> {
    http: &'http reqwest::Client,
}

#[derive(Deserialize)]
struct CalendarList {
    items: Vec<CalendarEntry>,
}

#[derive(Deserialize)]
struct CalendarEntry {
    id: String,
    /// Calendars the user hid in Google Calendar stay hidden in Tiki.
    selected: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventPage {
    items: Vec<GoogleEvent>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEvent {
    id: String,
    status: Option<String>,
    summary: Option<String>,
    location: Option<String>,
    start: GoogleTime,
    end: GoogleTime,
    hangout_link: Option<String>,
    conference_data: Option<ConferenceData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTime {
    date_time: Option<String>,
    /// All-day events have a date and no time.
    date: Option<NaiveDate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConferenceData {
    entry_points: Option<Vec<EntryPoint>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryPoint {
    entry_point_type: Option<String>,
    uri: Option<String>,
}

impl<'http> GoogleCalendar<'http> {
    const CALENDAR_LIST: &'static str =
        "https://www.googleapis.com/calendar/v3/users/me/calendarList";
    const EVENTS: &'static str = "https://www.googleapis.com/calendar/v3/calendars";

    pub fn new(http: &'http reqwest::Client) -> Self {
        Self { http }
    }

    /// Every meeting between `from` and `to` (local time) in the calendars the user shows.
    pub async fn meetings(
        &self,
        access_token: &str,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> TikiResult<Vec<Meeting>> {
        let list: CalendarList = self
            .get(access_token, Self::CALENDAR_LIST.to_owned())
            .await?;
        let mut meetings = Vec::new();
        for calendar in list
            .items
            .into_iter()
            .filter(|calendar| calendar.selected.unwrap_or(false))
        {
            meetings.extend(
                self.calendar_meetings(access_token, &calendar.id, from, to)
                    .await?,
            );
        }
        Ok(meetings)
    }

    async fn calendar_meetings(
        &self,
        access_token: &str,
        calendar_id: &str,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> TikiResult<Vec<Meeting>> {
        let mut meetings = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut url = Url::parse(Self::EVENTS).map_err(|error| Self::error(&error))?;
            url.path_segments_mut()
                .map_err(|()| Self::error(&"invalid calendar address"))?
                .push(calendar_id)
                .push("events");
            url.query_pairs_mut()
                .append_pair("timeMin", &Self::utc(from))
                .append_pair("timeMax", &Self::utc(to))
                .append_pair("singleEvents", "true")
                .append_pair("orderBy", "startTime")
                .append_pair("maxResults", "250");
            if let Some(token) = &page_token {
                url.query_pairs_mut().append_pair("pageToken", token);
            }
            let page: EventPage = self.get(access_token, url.to_string()).await?;
            meetings.extend(
                page.items
                    .into_iter()
                    .filter(|event| event.status.as_deref() != Some("cancelled"))
                    .filter_map(|event| Self::meeting(calendar_id, event)),
            );
            match page.next_page_token {
                Some(token) => page_token = Some(token),
                None => return Ok(meetings),
            }
        }
    }

    fn meeting(calendar_id: &str, event: GoogleEvent) -> Option<Meeting> {
        let (starts_at, all_day) = Self::moment(&event.start)?;
        let (ends_at, _) = Self::moment(&event.end)?;
        let video = event.conference_data.and_then(|data| {
            data.entry_points?
                .into_iter()
                .find(|point| point.entry_point_type.as_deref() == Some("video"))
                .and_then(|point| point.uri)
        });
        Some(Meeting {
            // One account can show the same event in two calendars: keep them apart.
            external_id: format!("{calendar_id}:{}", event.id),
            title: event.summary.unwrap_or_default(),
            starts_at,
            ends_at,
            all_day,
            location: event.location.unwrap_or_default(),
            join_url: event.hangout_link.or(video),
        })
    }

    fn moment(time: &GoogleTime) -> Option<(NaiveDateTime, bool)> {
        match (&time.date_time, time.date) {
            (Some(text), _) => Some((local_from_rfc3339(text)?, false)),
            (None, Some(day)) => Some((day.and_time(NaiveTime::MIN), true)),
            (None, None) => None,
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        url: String,
    ) -> TikiResult<T> {
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|error| Self::error(&error))?;
        let body = ProviderAnswer::body(response)
            .await
            .map_err(|error| Self::error(&error))?;
        serde_json::from_str(&body).map_err(|error| Self::error(&error))
    }

    fn utc(local: NaiveDateTime) -> String {
        Local
            .from_local_datetime(&local)
            .earliest()
            .map_or_else(|| local.and_utc(), |time| time.with_timezone(&Utc))
            .to_rfc3339()
    }

    fn error(error: &impl std::fmt::Display) -> TikiError {
        TikiError::Calendar(format!("Google calendar: {error}"))
    }
}
