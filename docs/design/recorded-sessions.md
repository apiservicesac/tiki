# Recorded sessions

Tiki records a work session (a call or a meeting) so the user can review it later: the media,
a local transcript and notes written by the AI. Nothing is recorded without the user's click.

## Decisions

| Topic | Decision |
|---|---|
| What is recorded | Chosen per session: audio + video, or audio only. Always transcribed. |
| When | The user starts and stops it (notch button, `Ctrl+Alt+R`). Tiki suggests it when a call starts (`CallDetector`) or a calendar meeting begins, once per call or meeting, never while recording. |
| Video | Windows.Graphics.Capture through `windows-capture` (MIT), H.264 + AAC in MP4 with the Media Foundation encoder. Normal preset: up to 1080p, 15 fps, 1.5 Mbps (~0.7 GB/h); high: 30 fps, 4 Mbps. Any monitor or one window. The island is content-protected so the notch stays out of the video. No ffmpeg (size and LGPL/GPL obligations). |
| Audio | `wasapi` (MIT): system loopback + microphone, aligned to 48 kHz and mixed for playback (AAC in the MP4, or Ogg Opus ~22 MB/h for audio only). Each source is also written as 16 kHz mono WAV for transcription and as crash insurance. |
| Speakers | Microphone and system audio are transcribed separately and labelled "You" and "Others". |
| Transcription | Local only: `whisper-rs` (whisper.cpp), CPU, after the session stops, in a persistent background queue at low priority. Default model `ggml-small-q5_1` (~181 MB, checksum pinned), downloaded the first time the user records; `base` (fast) and `large-v3-turbo` (precise) on offer. Language detected (es/en). |
| Notes | Generated automatically by the configured AI when the transcript is ready (a setting turns this off): summary, key points, decisions and proposed tasks applied with one click. Without an AI key: transcript only. |
| Storage | `sessions`, `transcript_segments` (+ FTS5 search), `session_notes`, `session_action_items`, `transcription_jobs`. Files in `app_data/sessions/<date>-<id>/`. Media deleted after 90 days by default; transcript and notes kept. |
| Consent | Every suggestion and every start reminds the user to tell the participants. A red dot and timer stay visible while recording. |

## Layout

- `crates/tiki-core/src/sessions/`: `Session`, `RecordingMode`, `SessionStatus` with checked transitions, `SessionClock` (timestamps minus paused time), `Transcript`, `SessionNotes`, `MeetingLink`, `RetentionPolicy`, `RecordingSuggestionPolicy`, `SessionService`.
- `crates/tiki-core/src/storage/session_repository.rs` and the `SESSIONS` migration.
- `crates/tiki-recording/` (new crate, keeps heavy dependencies out of the core): traits `AudioSource`, `VideoSource`, `MediaSink`, `Transcriber`, `ModelStore`; the cross-platform `AudioMixer`, WAV and Opus writers, `RecordingPipeline`, model downloader and transcription worker; Windows sources and the MP4 sink behind `cfg(windows)`; Whisper behind a feature.
- Shell: `session_recorder.rs`, `transcription_runner.rs`, `commands/session_commands.rs`, new shell events, the shortcut in `DesktopConfig`.
- Interface: `recording` mode in `IslandModeMachine`, `RecordingIndicator`, `RecordingPrompt`, a Sessions tab (list, player with the transcript synced to time, search, delete, export), `SessionNotesAdvisor`.

## Steps

1. Domain and migration (core, tested in Docker).
2. Suggestion policy and prompt in the notch.
3. Recording pipeline with fake sources and sinks: mixing, silence fill, drift, pause.
4. Windows audio-only recording (checked in GitHub Actions).
5. Local transcription with the model download.
6. Sessions tab.
7. AI notes.
8. Video.
9. Recovery after a crash, export and disk usage.

## Risks

Windows N editions without Media Foundation; GPUs without a hardware encoder (software fallback);
echo when the user is on speakers; an unfinished MP4 after a crash (the WAV survives); whisper.cpp
build time in CI; CPU load after long meetings.
