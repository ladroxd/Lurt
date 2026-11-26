# Changelog

All notable changes to Lurt are listed here, newest first.

## [Unreleased]

### Added
### Changed
### Fixed

## [0.1.0] - 2026-11-26

### Added

- Borderless liquid-glass window (blur, acrylic or mica backdrop) with dark, light and system themes, accent colors, roundness and a compact layout.
- The blur behind the window is tinted dark in dark mode and light in light mode, so low background opacity shows the desktop without washing the UI out to gray.
- Background opacity and window opacity (fades the whole window, websites included) sliders. Window opacity previews live while you drag.
- Google account button at the top right. It shows the account you're signed in to Google with in Lurt (photo or initial), and opens a card with Manage your Google Account, Switch account, Add another account and Sign out. When you're not signed in it offers Sign in to Google. It updates after you sign in or out on a Google page.
- Account section in Settings: your Google account with the same actions, what Lurt saves on this PC (settings, bookmark, folder and history counts, cookies and sign-ins, database size and folder), and a Lurt profile backup that saves your settings to a file and loads them back. Loading only applies settings Lurt knows and refuses files without any.
- Delete all browser data, in Settings > Account and Privacy: history, bookmarks and folders, saved passwords and autofill, cookies, sign-ins, cache and site data, settings, start page favourites and open tabs. If you're signed in to Google it signs you out first, and you type "delete" to confirm. Files in the mods folder stay.
- The startup skeleton matches the interface 1:1: four toolbar buttons, the window buttons, the restored tabs, the bookmarks bar with your bookmarks, and the start page clock, search pill and tiles in their real positions. Sizes that depend on your data (tile labels, bookmarks, the clock) are measured while Lurt runs and saved to `skeleton.json` for the next launch.
- Lurt logo: two clear-glass bars crossing into an L, used as the app, window and taskbar icon and on the About page.
- Maximizing keeps the same glass color as a normal window. Windows 10 paints the blur of a truly maximized window with a light gray fill, so Lurt maximizes by filling the screen's work area itself (button, double-click, Win+Up and dragging to the top all work, and dragging the toolbar restores it).
- The Google account button sees the account you signed in with. It read Google's account list with the wrong fields and dropped valid accounts, so it kept saying "Sign in" and sent you to your Google Account page instead.
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

[Unreleased]: https://github.com/ladroxd/Lurt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ladroxd/Lurt/releases/tag/v0.1.0
