use std::thread;

use tauri::{AppHandle, Manager};
use tiki_core::calendar::{AccountId, CalendarAccount, CalendarProvider};
use tiki_core::i18n::Translator;
use tiki_core::{TikiError, TikiResult};

use super::google_calendar::GoogleCalendar;
use super::microsoft_calendar::MicrosoftCalendar;
use super::oauth_client::OAuthClient;
use super::profile_reader::ProfileReader;
use super::sign_in::SignIn;
use super::sign_in_page::SignInPages;
use super::token_store::TokenStore;
use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// Connects calendar accounts and keeps the core's copy of their meetings fresh.
pub struct CalendarSync {
    http: reqwest::Client,
}

impl Default for CalendarSync {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

impl CalendarSync {
    /// Providers this build can connect (it has their app registration).
    pub fn available() -> Vec<CalendarProvider> {
        CalendarProvider::ALL
            .into_iter()
            .filter(|provider| OAuthClient::of(*provider).is_some())
            .collect()
    }

    /// Signs in in the browser, remembers the account and copies its meetings.
    pub async fn connect(
        &self,
        app: &AppHandle,
        provider: CalendarProvider,
    ) -> TikiResult<CalendarAccount> {
        let client = OAuthClient::of(provider).ok_or_else(|| {
            TikiError::Calendar("this build cannot connect that calendar".to_owned())
        })?;
        let state = app.state::<AppState>();
        let pages = state.with_workspace(|workspace| {
            Ok(SignInPages::new(&Translator::new(
                workspace.preferences().locale,
            )?))
        })?;
        let signed_in = SignIn::new(self.http.clone())
            .sign_in(app, client, pages)
            .await?;
        let account_id = state
            .with_workspace(|workspace| workspace.connect_calendar(provider, &signed_in.email))?;
        TokenStore::save(account_id, &signed_in.refresh_token)?;
        let account = state.with_workspace(|workspace| workspace.calendar_account(account_id))?;
        // Signing in again may grant what an older sign-in lacked (Google's `profile`).
        self.sync_account(app, &account, true).await?;
        ShellEvent::CalendarSynced.emit(app);
        Ok(account)
    }

    /// Forgets an account: its sign-in and its meetings.
    pub fn disconnect(app: &AppHandle, account_id: AccountId) -> TikiResult<()> {
        TokenStore::remove(account_id)?;
        app.state::<AppState>()
            .with_workspace(|workspace| workspace.disconnect_calendar(account_id))?;
        ShellEvent::CalendarSynced.emit(app);
        Ok(())
    }

    /// Copies the meetings of every account, and its name and picture once a day; one
    /// failing account does not stop the others.
    pub async fn sync_all(&self, app: &AppHandle) {
        let (accounts, now) = app
            .state::<AppState>()
            .with_workspace(|workspace| Ok((workspace.calendar_accounts()?, workspace.now())))
            .unwrap_or_default();
        for account in &accounts {
            let identity_due = account.identity_due(now, DesktopConfig::ACCOUNT_IDENTITY_REFRESH);
            if let Err(error) = self.sync_account(app, account, identity_due).await {
                tracing::warn!(account = account.account_id.0, %error, "could not sync the calendar");
            }
        }
        if !accounts.is_empty() {
            ShellEvent::CalendarSynced.emit(app);
        }
    }

    /// Syncs every few minutes for the rest of the session.
    pub fn start(app: AppHandle) {
        thread::spawn(move || {
            let sync = Self::default();
            loop {
                tauri::async_runtime::block_on(sync.sync_all(&app));
                thread::sleep(DesktopConfig::CALENDAR_SYNC_INTERVAL);
            }
        });
    }

    async fn sync_account(
        &self,
        app: &AppHandle,
        account: &CalendarAccount,
        identity_due: bool,
    ) -> TikiResult<()> {
        let client = OAuthClient::of(account.identity.provider).ok_or_else(|| {
            TikiError::Calendar("this build cannot read that calendar".to_owned())
        })?;
        let refresh_token = TokenStore::refresh_token(account.account_id)?
            .ok_or_else(|| TikiError::Calendar("the account needs to sign in again".to_owned()))?;
        let tokens = SignIn::new(self.http.clone())
            .refresh(client, &refresh_token)
            .await?;
        if let Some(rotated) = &tokens.refresh_token {
            TokenStore::save(account.account_id, rotated)?;
        }
        let state = app.state::<AppState>();
        let (from, to) = state.with_workspace(|workspace| Ok(workspace.calendar_window()))?;
        let meetings = match account.identity.provider {
            CalendarProvider::Microsoft => {
                MicrosoftCalendar::new(&self.http)
                    .meetings(&tokens.access_token, from, to)
                    .await?
            }
            CalendarProvider::Google => {
                GoogleCalendar::new(&self.http)
                    .meetings(&tokens.access_token, from, to)
                    .await?
            }
        };
        tracing::info!(
            account = account.account_id.0,
            meetings = meetings.len(),
            "calendar synced"
        );
        state
            .with_workspace(|workspace| workspace.store_meetings(account.account_id, &meetings))?;
        if identity_due
            && let Err(error) = self
                .refresh_identity(app, account, &tokens.access_token)
                .await
        {
            tracing::warn!(account = account.account_id.0, %error, "could not read the account's name and picture");
        }
        Ok(())
    }

    /// Asks the provider for the account's name and picture and keeps them.
    async fn refresh_identity(
        &self,
        app: &AppHandle,
        account: &CalendarAccount,
        access_token: &str,
    ) -> TikiResult<()> {
        let profile = ProfileReader::new(&self.http)
            .read(account.identity.provider, access_token)
            .await?;
        app.state::<AppState>().with_workspace(|workspace| {
            workspace.update_calendar_identity(
                account.account_id,
                &profile.display_name,
                profile.avatar.as_deref(),
            )
        })
    }
}
