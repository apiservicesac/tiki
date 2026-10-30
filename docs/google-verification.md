# Google OAuth verification

What to enter in Google Cloud Console › APIs & Services › OAuth consent screen (Branding,
Audience and Data access), and how to prepare the demo video. Tiki asks for one sensitive
scope, `calendar.readonly`, so the review needs no security assessment.

## App information

| Field | Value |
|---|---|
| App name | Tiki |
| User support email | principal@apiservicesac.com |
| App logo | `docs/assets/tiki-logo-120.png` (120×120 PNG; Google does not take SVG) |
| Application home page | https://tiki.apiservicesac.com/ |
| Privacy policy | https://tiki.apiservicesac.com/privacy-policy.html (the policy page itself, not the home page) |
| Authorized domain | `apiservicesac.com` |
| Developer contact | principal@apiservicesac.com |
| Publishing status | In production |

## Serve the site from apiservicesac.com

Google rejects `github.io` addresses: the policy must live on a domain the developer owns.
The site stays on GitHub Pages under the subdomain `tiki.apiservicesac.com`.

1. At the DNS provider of `apiservicesac.com`, add a `CNAME` record: name `tiki`, value
   `apiservicesac.github.io`.
2. Add the file `docs/CNAME` containing `tiki.apiservicesac.com`, push it, and turn on
   **Enforce HTTPS** in the repository's Settings › Pages once the certificate is ready.
3. In Google Search Console add a **Domain** property for `apiservicesac.com` and verify it
   with the `TXT` record it gives, at the same DNS provider.
4. In the OAuth consent screen add `apiservicesac.com` under Authorized domains, and set the
   home page and privacy policy to the addresses in the table above.

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
