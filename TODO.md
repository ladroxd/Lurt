# TODO

What's still pending for Lurt. Finished work moves to [CHANGELOG.md](CHANGELOG.md).

## Release

- [x] Push `main` up to the Google account button (4442139).
- [x] Push the skeleton, Delete all browser data, Chrome import, branded loading screen, Fewer processes and Privacy page commits.
- [x] Push the downloads button, Downloads page and the built-in extensions plan (d214c6a).
- [ ] For each new version: bump `version` in `Cargo.toml`, add `.github/release-notes/vX.Y.Z.md` (from `TEMPLATE.md`), update the version in `assets/readme/version.svg` and `assets/readme/download.svg`, push `main`.

## Trust and antivirus

- [ ] Find out which antivirus flags `lurt.exe` and the detection name, then report it to that vendor as a false positive (Microsoft: https://www.microsoft.com/en-us/wdsi/filesubmission). Repeat for new releases while unsigned.
- [ ] Add an open source license (needed for free signing).
- [ ] Code-sign releases: apply to SignPath Foundation (free for open source, https://signpath.org/apply) and sign in the release workflow, or buy a certificate.

## Updates

- [ ] **Auto updater.** Check GitHub releases for a newer version than the one running (at startup, then every few hours, and from a "Check for updates" button in Settings > About). Compare versions properly (0.10.0 is newer than 0.9.0), skip drafts, and verify the download against the release's `lurt.exe.sha256` before using it.
- [ ] Download the new `lurt.exe` in the background with progress, then offer "Restart to update" (or update on the next launch). Swap the running exe safely: rename the old one, move the new one in, restart, and roll back if the new version fails to start.
- [ ] Settings > About: current version, last checked time, "Check for updates", and a switch for automatic updates (on by default).
- [ ] **What's new window.** After an update, the first launch shows a styled glass window with what changed in that version: New, Changed and Fixed sections taken from the release notes, with headlines and short descriptions. It shows once per version and never again until the next update (remember the last version shown in the profile). Also reachable from Settings > About ("What's new").
- [ ] If several versions were skipped, the window lists each one's changes, newest first.
- [ ] Needs the repo (or a separate public update feed) to be public: an app can't read private releases without a token, which must never be built into Lurt.

## Account

- [ ] **Sync Lurt settings to the Google account.** Store settings (and later bookmarks) in the account's Google Drive app data folder, which only Lurt can see. Needs a Google Cloud project with an OAuth client ID for a desktop app (owner to create it), sign-in through the system browser with PKCE, the `drive.appdata` scope, and a "Sync now / last synced" row in Settings > Account. Until then, Settings > Account saves and loads settings as a file.
- [ ] Delete all browser data: optionally keep chosen kinds (for example keep bookmarks, delete the rest).
- [ ] Show the account photo even before Google's photo server is reachable (cache it in the profile).

## Privacy

- [ ] Per-site exceptions for fingerprint protection and HTTPS upgrades (like the paused-sites list for blocking).
- [ ] Remember site permission choices per site, with a list in Settings to review and reset them.
- [ ] A private window (all tabs private, its own window) besides private tabs.
- [ ] Custom filter lists: add a list by URL, and your own rules.
- [ ] Custom Secure DNS server (any DoH template), and an "automatic" mode that falls back to Windows DNS.
- [ ] Block third-party cookies on their own, separate from Strict tracking prevention.
- [ ] Show in the shield panel what each protection did on the page (links cleaned, HTTPS upgrade, pings blocked).

## Extensions

- [ ] **Chrome extension support.** Load Chrome extensions (Manifest V3) through WebView2's browser extension API (`AreBrowserExtensionsEnabled`, `ICoreWebView2Profile7::AddBrowserExtension`). Check which APIs WebView2 supports and show a clear message for extensions that need unsupported ones.
- [ ] **Extensions page ("workshop").** A page listing installed extensions with on/off switches, details, permissions and remove, styled like Settings.
- [ ] **Developer mode.** A switch on the extensions page that reveals "Load unpacked": pick a folder with a `manifest.json` and Lurt loads it, with a reload button for development.
- [ ] **Built-in extensions.** Bundle the owner's own extensions with Lurt, turned off by default and enabled from the extensions page.
- [ ] **Built-in: Video & audio downloader.** A download button on YouTube (and other video sites it supports) to save as MP3, M4A, OPUS, WAV or FLAC audio, or MP4, WEBM or MKV video. Pick the quality (up to 4K and 60 fps), download whole playlists, and choose subtitles and the thumbnail. Built on yt-dlp and ffmpeg, downloaded on first use rather than bundled, with progress in the downloads panel. Only for videos you're allowed to download.
- [ ] **Built-in: PDF tools.** In Lurt's PDF viewer: free draw with pen, highlighter and eraser (colors and sizes), text boxes, shapes and arrows, sticky notes, highlight and underline text, sign with a drawn or typed signature, fill forms, and save the edited PDF or export pages as images. Undo and redo, and annotations stay editable after saving.
- [ ] **Built-in: Screenshot tool.** Capture the visible page, the full scrolling page or a selected area, then draw on it, blur parts out and copy or save it.
- [ ] **Built-in: Picture-in-picture.** A button on any video to pop it out into a small always-on-top window.
- [ ] **Built-in: Dark mode for every site.** Darkens sites that have no dark theme, with a per-site switch.
- [ ] **Built-in: Translate page.** Translate the whole page or a selection in place.
- [ ] **Built-in: Reader mode.** Shows just the article text with adjustable font, size and width, and read aloud.
- [ ] **Built-in: Color picker and page tools.** Pick any color on a page, measure elements, and see fonts used.
- [ ] Extension toolbar buttons (pinned icons next to the shield) and their popups.
- [ ] Install from the Chrome Web Store, if WebView2 allows it, or from a `.crx` / `.zip` file.

## Browsing

- [ ] Drag and drop files from Explorer onto the window to open them (needs the WebView2 file path from `postMessageWithAdditionalObjects`).
- [ ] "Open file…" (Ctrl+O) with a Windows file picker.
- [ ] Register Lurt as a handler for `.pdf`, `.html` and image files ("Open with").
- [ ] Downloads page: select several downloads and remove or delete their files at once; delete the file itself from the list.
- [ ] Find in page (Ctrl+F) styled like the rest of the UI.
- [ ] Zoom controls (Ctrl+plus, Ctrl+minus, Ctrl+0) with a zoom indicator.

## Startup

- [ ] Compact layout: measure the one-row toolbar like the normal layout so its skeleton matches 1:1 too (the normal layout already does).
- [ ] Skeleton for a restored website tab: draw that site's real outline (from a saved low-res snapshot) instead of a generic page.

## Bookmarks

- [ ] Bookmark manager page: search, move many at once, sort, export to an HTML file.

## Check by hand

These work in tests and screenshots but need a real mouse and keyboard to confirm:

- [ ] Address bar: click once selects the whole URL; drag to highlight part of it.
- [ ] Waking a sleeping tab shows the blurred picture of the page.
- [ ] Dragging bookmarks between the bar and open folder menus on a real website (the page is swapped for a snapshot while menus are open).
- [ ] Signed in to Google in Lurt: the account button shows your photo and name; Sign out signs you out of Gmail and YouTube too.
- [ ] Delete all browser data while signed in: it signs you out first, then deletes everything after you type "delete".
- [ ] Second launch after opening the start page: the skeleton lines up with your tiles, bookmarks and clock.
- [ ] Protection level Strict on everyday sites (YouTube, Google, banking): note any that break.
- [ ] WebRTC IP protection with a VPN on: a WebRTC leak test shows only the VPN address.
- [ ] Secure DNS with Quad9, Mullvad or Google (the Cloudflare test page only detects Cloudflare).
- [ ] Private tab: sign in to a site, close every private tab, open a new one: you're signed out.
- [ ] Opening YouTube or Twitch shows the tinted, breathing loading screen with just the site name.
- [ ] Downloads panel on real sites: download a big file, pause, resume, open it and show it in its folder.
- [ ] Download settings: Change… folder picker, Ask where to save (the Save as window, and cancelling it), and an "Open when finished" rule.
