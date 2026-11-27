# Changelog

All notable changes to Lurt are listed here, newest first.

## [Unreleased]

### Added
### Changed
### Fixed

## [0.8.8] - 2026-11-27

### Added

- Tab volume: a speaker on tabs that make sound (click to mute), and a toolbar mixer with a volume slider and mute for each of them, on top of the site's own volume.

### Fixed

- Sites like Discord showed a colored letter instead of their icon on the bookmarks bar, on restored or sleeping tabs, and in history, suggestions and the favourites list.

## [0.8.7] - 2026-11-27

### Added

- Page zoom: Ctrl + scroll (also Ctrl + / -, Ctrl 0 and touchpad pinch) zooms websites, and Settings > Appearance > Page zoom shows and sets it. One zoom level for all sites, remembered between launches.
- MIT license, a privacy policy and a code signing policy.
- Microsoft Store packaging (`packaging/msix.ps1`, see `packaging/STORE.md`); the release workflow builds the package once the Store identity is set.

## [0.8.6] - 2026-11-27

### Added

- Drag tabs along the strip to reorder them. The order is kept between launches.

### Changed

- The start page's search box is now a real search box: you type in it, with suggestions under it, instead of being sent to the address bar.
- The add favourite button is a smaller round button centered under the favourites, with a hover effect.
- Favourites are centered on the start page, also when there are only a few.

### Fixed

- lurt.exe now carries its name, publisher, description and version (right-click > Properties > Details). Without them it looked anonymous to Windows and antivirus scanners. It is still unsigned.

## [0.8.5] - 2026-11-27

### Added

- Borderless liquid-glass window (blur, acrylic or mica backdrop) with dark, light and system themes, accent colors, roundness and a compact layout.
- The blur behind the window is tinted dark in dark mode and light in light mode, so low background opacity shows the desktop without washing the UI out to gray.
- Background opacity and window opacity (fades the whole window, websites included) sliders. Window opacity previews live while you drag.
- Google account button at the top right. It shows the account you're signed in to Google with in Lurt (photo or initial), and opens a card with Manage your Google Account, Switch account, Add another account and Sign out. When you're not signed in it offers Sign in to Google. It updates after you sign in or out on a Google page.
- Account section in Settings: your Google account with the same actions, what Lurt saves on this PC (settings, bookmark, folder and history counts, cookies and sign-ins, database size and folder), and a Lurt profile backup that saves your settings to a file and loads them back. Loading only applies settings Lurt knows and refuses files without any.
- Downloads: a downloads button next to the shield, with a progress ring (and a count when several run) and a little bounce when a download starts. Its panel lists your downloads with size, speed and time left, and lets you pause, resume, cancel, open a file, show it in its folder, try a failed one again, remove it from the list or clear the list, and open the Downloads folder. Downloads no longer navigate away from the page or show the engine's own pop-up. The list is kept between launches (not for private tabs), and files that were moved or deleted show as such.
- Choose where a bookmark goes: the star (or Ctrl+D) saves the page and opens a card under it to rename it, pick its folder from the whole folder tree, or make a new folder right there, and Remove. On a bookmarked page the star opens the same card to edit it. New bookmarks go to the folder you used last.
- Bookmark folder menus no longer cover the add or edit window or the right-click menu (Edit, Copy link, Delete) of a bookmark inside a folder.
- Releases: a draft GitHub release with lurt.exe is made automatically for each new version in Cargo.toml when main is pushed (or for a pushed version tag), with notes from .github/release-notes or the CHANGELOG. The README has a Download for Windows button.
- A Downloads page (Show all downloads at the bottom of the downloads panel, Ctrl+J, or Settings > Downloads), like chrome://downloads: every download grouped by day, with search, the link it came from, open, show in folder, retry and remove, Open folder and Clear list. Its Download settings tab sets the download folder (with a Windows folder picker), Ask where to save each file, and rules per file type: allow, ask where to save, open when finished, or block (with a one-click "Block programs and scripts"). Blocked downloads show in the list with the reason.
- A full Privacy & trackers settings page:
  - Protection levels: Standard (ads, trackers and malware blocked, sites work as usual) or Strict (every list, strict tracking prevention, fingerprint and IP protection). Changing anything below shows Custom.
  - Pick your filter lists: EasyList, EasyPrivacy, Peter Lowe's, uBlock filters, Malware & scams, Cookie banners, Annoyances and Crypto miners. The engine rebuilds when you change them.
  - Clean links: utm_, fbclid, gclid and other tracking tags are removed from addresses before they load, plus share trackers like YouTube's si.
  - Global Privacy Control and Do Not Track are sent with every request.
  - Link-click pings (hyperlink auditing) are blocked.
  - Resist fingerprinting (in Strict): sites see a generic graphics card, 4 cores and 8 GB, and canvas and audio reads get invisible noise that is different for every site.
  - HTTPS upgrades: http:// sites load over HTTPS, falling back to HTTP for sites that don't support it.
  - Secure DNS through Cloudflare, Quad9, Mullvad or Google.
  - WebRTC IP protection, so calls don't reveal local addresses or get around a VPN.
  - Site permissions: always block location, camera, microphone or notifications, or keep asking.
  - Private tabs (Ctrl+Shift+N, or right-click the + button): their own cookies, gone when the last private tab closes, no history, not restored or reopened. Links they open stay private. They have a purple tint and a banner on their start page. You can also choose to always open private tabs.
  - Delete history, or cookies, site data and cache, when Lurt closes.
- "Clear cookies & cache" no longer also deletes saved passwords and autofill.
- Favourites and bookmarks show the right icon for sites that don't have a /favicon.ico, like Discord. Lurt remembers the icon a page declares, or reads it from the site's homepage.
- The loading screen takes on the site's colors: the background is tinted with the color of the site's logo (YouTube red, Twitch purple…) and breathes while the page loads, the spinner ring follows it, and the text is just the site's name ("Youtube", "Twitch") instead of "Waking up youtube.com". Well-known sites have their brand color built in; for others Lurt picks the strongest color of the site's icon.
- Fewer processes (Settings > Performance, on by default): embedded frames from other sites, like ads, payment and video embeds, share their page's process instead of getting one each. With YouTube, Twitch and The Verge open this went from 35 engine processes and 2.3 GB to 10 processes and 1.2 GB. Turn it off for stricter isolation between sites; it applies after a restart.
- Importing a Chrome profile also brings the shortcuts from its new tab page in as start page favourites (skipping ones you already have).
- Importing from Chrome also reads `AccountBookmarks`, where newer Chrome keeps the bookmarks of a Google account that is signed in without full sync. Those bookmarks were missed before.
- Bring your data in after signing in: the first time a Google account shows up, Lurt offers to import bookmarks and history from the browser profile on this PC signed in with the same account (other profiles are listed too). Google doesn't let other browsers download Chrome Sync, so this copies from the local profile; a Google Takeout export still works from Settings > Your data.
- Delete all browser data, in Settings > Account and Privacy: history, bookmarks and folders, saved passwords and autofill, cookies, sign-ins, cache and site data, settings, start page favourites and open tabs. If you're signed in to Google it signs you out first, and you type "delete" to confirm. Files in the mods folder stay.
- The startup skeleton matches the interface 1:1: four toolbar buttons, the window buttons, the restored tabs, the bookmarks bar with your bookmarks, and the start page clock, search pill and tiles in their real positions. Sizes that depend on your data (tile labels, bookmarks, the clock) are measured while Lurt runs and saved to `skeleton.json` for the next launch.
- Lurt logo: two clear-glass bars crossing into an L, used as the app, window and taskbar icon and on the About page.
- Maximizing keeps the same glass color as a normal window. Windows 10 paints the blur of a truly maximized window with a light gray fill, so Lurt maximizes by filling the screen's work area itself (button, double-click, Win+Up and dragging to the top all work, and dragging the toolbar restores it).
- The Google account button sees the account you signed in with. Lurt asked Google for the account list with a GET, which Google refuses (it wants a POST, like Chrome sends), and read the list with the wrong fields, so it kept saying "Sign in" and sent you to your Google Account page instead.
- After Delete all browser data, the start page favourites disappear right away instead of showing the old tiles until Lurt restarts.
- Right-clicking the bookmarks bar works while a website is open. The menu used to close as soon as it opened, because swapping the page for its snapshot moves keyboard focus for a moment, and a redraw of the bar in between could make the click miss.
- After a maximize, restore or resize, the window is re-presented once resizing settles, so the Windows 10 blur covers the whole window instead of leaving a gray rectangle at the old size.
- Native startup skeleton that appears in about 30 ms while WebView2 starts.
- Waking a sleeping tab shows a blurred picture of how you left it (or a soft accent glow) with a spinning glass ring around the site's icon, instead of a white flash.
- Tabs that share one engine process, open lazily, and sleep when idle.
- Address bar with bookmark and history suggestions over a blurred snapshot of the page.
- Ad and tracker blocking (EasyList, EasyPrivacy, Peter Lowe's list) plus WebView2 tracking prevention.
- Settings window: a centered glass window over a blurred view of the browser, with sections for Appearance, Start page, Bookmarks, Browsing, Privacy & trackers, Your data, Performance, Mods and About. Open it with the gear button or Ctrl+,.
- Settings search: find any setting by name, description or option (Ctrl+F while Settings is open); picking a result jumps to it and highlights it.
- Trackers tab in the side panel (next to Bookmarks and History): blocking switches, what was blocked on the page, paused sites and tracking prevention.
- Start page favourites: add your own sites with the + tile, and right-click any tile to open, pin, edit or remove it. Removed sites can be brought back in Settings → Start page.
- Shield popover: click the blocked count to turn blocking off or on for the site and see which domains were blocked and what kind of requests.
- Bookmarks bar under the toolbar: bookmarks and folders in your own order, folders open as menus with cascading subfolders, a » menu for what doesn't fit, drag to reorder or into folders, drop links onto it, and right-click to open, edit, rename, add folders or delete. Show it always, only on the start page, or hide it (Ctrl+Shift+B).
- Imported bookmarks keep their folders: Chrome's bookmarks bar lands on Lurt's bar, and "Other bookmarks" and "Mobile bookmarks" become folders on it.
- History management: select pages to delete (click the icons, Shift+click for a range), clear the last hour, day, week, four weeks or everything, and delete old history automatically after a day, a week, a month, three months, a year or a custom number of days.
- Open local files: type or paste a path like `C:\Users\me\file.pdf` (quotes from "Copy as path" are fine) to open images, PDFs, HTML and other files in the browser.
- The side panel (bookmarks, history, trackers) closes when you click anywhere else, the page included.
- Bookmarks, history and session restore.
- Import of bookmarks and history from Chromium browsers on the PC and from Google Takeout files.
- Password saving and form autofill settings.
- UI and per-site CSS/JS mods.
- Google as the default search engine.

[Unreleased]: https://github.com/ladroxd/Lurt/compare/v0.8.5...HEAD
[0.8.5]: https://github.com/ladroxd/Lurt/releases/tag/v0.8.5
