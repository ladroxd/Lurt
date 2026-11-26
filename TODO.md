# TODO

What's still pending for Lurt. Finished work moves to [CHANGELOG.md](CHANGELOG.md).

## Release

- [x] Push `main` up to the Google account button (4442139).
- [ ] Owner tests the latest build and says "push" for the newer commits (skeleton, Delete all browser data, bookmarks bar right-click fix).
- [ ] Push `main`, then tag `v0.1.0` and push the tag. The release workflow builds `lurt.exe` and opens a **draft** release with the notes from `.github/release-notes/v0.1.0.md`.
- [ ] Review the draft release on GitHub and publish it.

## Account

- [ ] **Sync Lurt settings to the Google account.** Store settings (and later bookmarks) in the account's Google Drive app data folder, which only Lurt can see. Needs a Google Cloud project with an OAuth client ID for a desktop app (owner to create it), sign-in through the system browser with PKCE, the `drive.appdata` scope, and a "Sync now / last synced" row in Settings > Account. Until then, Settings > Account saves and loads settings as a file.
- [ ] Delete all browser data: optionally keep chosen kinds (for example keep bookmarks, delete the rest).
- [ ] Show the account photo even before Google's photo server is reachable (cache it in the profile).

## Extensions

- [ ] **Chrome extension support.** Load Chrome extensions (Manifest V3) through WebView2's browser extension API (`AreBrowserExtensionsEnabled`, `ICoreWebView2Profile7::AddBrowserExtension`). Check which APIs WebView2 supports and show a clear message for extensions that need unsupported ones.
- [ ] **Extensions page ("workshop").** A page listing installed extensions with on/off switches, details, permissions and remove, styled like Settings.
- [ ] **Developer mode.** A switch on the extensions page that reveals "Load unpacked": pick a folder with a `manifest.json` and Lurt loads it, with a reload button for development.
- [ ] **Built-in extensions.** Bundle the owner's own extensions with Lurt, turned off by default and enabled from the extensions page.
- [ ] Extension toolbar buttons (pinned icons next to the shield) and their popups.
- [ ] Install from the Chrome Web Store, if WebView2 allows it, or from a `.crx` / `.zip` file.

## Browsing

- [ ] Drag and drop files from Explorer onto the window to open them (needs the WebView2 file path from `postMessageWithAdditionalObjects`).
- [ ] "Open file…" (Ctrl+O) with a Windows file picker.
- [ ] Register Lurt as a handler for `.pdf`, `.html` and image files ("Open with").
- [ ] Downloads: a downloads panel with progress, open and show in folder.
- [ ] Find in page (Ctrl+F) styled like the rest of the UI.
- [ ] Zoom controls (Ctrl+plus, Ctrl+minus, Ctrl+0) with a zoom indicator.

## Startup

- [ ] Compact layout: measure the one-row toolbar like the normal layout so its skeleton matches 1:1 too (the normal layout already does).
- [ ] Skeleton for a restored website tab: draw that site's real outline (from a saved low-res snapshot) instead of a generic page.

## Bookmarks

- [ ] Bookmark manager page: search, move many at once, sort, export to an HTML file.
- [ ] Star button: choose the folder when adding a bookmark.

## Check by hand

These work in tests and screenshots but need a real mouse and keyboard to confirm:

- [ ] Address bar: click once selects the whole URL; drag to highlight part of it.
- [ ] Waking a sleeping tab shows the blurred picture of the page.
- [ ] Dragging bookmarks between the bar and open folder menus on a real website (the page is swapped for a snapshot while menus are open).
- [ ] Signed in to Google in Lurt: the account button shows your photo and name; Sign out signs you out of Gmail and YouTube too.
- [ ] Delete all browser data while signed in: it signs you out first, then deletes everything after you type "delete".
- [ ] Second launch after opening the start page: the skeleton lines up with your tiles, bookmarks and clock.
