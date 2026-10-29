# Google OAuth verification

What to enter in Google Cloud Console › APIs & Services › OAuth consent screen (Branding,
Audience and Data access), and how to prepare the demo video. Tiki asks for one sensitive
scope, `calendar.readonly`, so the review needs no security assessment.

## App information

| Field | Value |
|---|---|
| App name | Tiki |
| User support email | principal@apiservicesac.com |
| App logo | `apps/desktop/assets/tiki-icon.svg` exported as a 120×120 PNG |
| Application home page | https://apiservicesac.github.io/tiki/ |
| Privacy policy | https://apiservicesac.github.io/tiki/privacy-policy.html |
| Authorized domain | `apiservicesac.github.io` |
| Developer contact | principal@apiservicesac.com |
| Publishing status | In production |

## Verify the domain first

1. Open Google Search Console and add a **URL prefix** property: `https://apiservicesac.github.io/tiki/`.
2. Choose **HTML tag** and copy the `<meta name="google-site-verification" …>` line.
3. Add that line inside `<head>` of `docs/index.html`, commit and push to `main`.
4. Wait a minute for GitHub Pages and press **Verify** in Search Console.
5. Back in the OAuth consent screen, add `apiservicesac.github.io` under Authorized domains.

Use the same Google account in Search Console and in Cloud Console.

## Scope justification

Scope: `https://www.googleapis.com/auth/calendar.readonly`

> Tiki is a desktop work assistant for Windows. It reads the user's calendar events (title,
> start and end time, location and video-call link) to list them in its Agenda next to the
> user's tasks and reminders, and to show an alert 15 minutes before each meeting with a
> button to join the call. Tiki never creates, edits or deletes events. Event data is stored
> only in a local database on the user's computer; it is not sent to any server owned by the
> developer, not shared, and not used for advertising. A narrower scope is not enough
> because Tiki needs the events of every calendar the user has, not only free/busy times.

## Demo video (2–3 minutes, unlisted on YouTube)

Record the screen in English, with the browser address bar visible.

1. Open Tiki and go to Advanced settings › Mail and calendar.
2. Press **Connect Google**. Show the Google consent screen: the app name "Tiki", the client
   ID in the address bar, and the calendar permission ("See and download any calendar you
   can access using your Google Calendar").
3. Accept and show the page that confirms the connection, then go back to Tiki.
4. Open the Agenda tab and show today's meetings coming from Google Calendar.
5. Show a meeting alert in the notch with its **Join** button.
6. Back in Mail and calendar, press **Disconnect** to show the user can revoke access.

## After submitting

Google answers by email to the developer contact. If they ask for changes, they usually
concern the privacy policy wording or the video; update them and reply in the same thread.
