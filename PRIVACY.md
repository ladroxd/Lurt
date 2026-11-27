# Privacy

Lurt has no servers, accounts, analytics or telemetry. Its developer receives nothing from your copy.

## What stays on your PC

History, bookmarks, settings, open tabs and the downloads list are stored in your Windows user profile (`%APPDATA%\Lurt`). Cookies, cache, saved passwords and form data are kept by the WebView2 engine in `%LOCALAPPDATA%\Lurt`. **Settings → Account** shows what is saved and can delete all of it.

Importing from another browser reads that browser's bookmarks and history on your PC, only when you ask. Passwords and cookies are not imported.

## What Lurt connects to

- **Websites you open**, and your chosen search engine when you search.
- **Filter list hosts** (easylist.to, pgl.yoyo.org, ublockorigin.github.io, secure.fanboy.co.nz, raw.githubusercontent.com), to download the ad and tracker lists you have turned on. Lists are refreshed every few days.
- **Site icons:** Lurt may request a site's icon or home page to show it on the start page, bookmarks and loading screen.
- **Google, if you sign in to Google in Lurt:** Lurt asks `accounts.google.com`, with your Google cookies, which accounts are signed in, to show your name and photo. Nothing is sent anywhere else.
- **Secure DNS**, only if you turn it on: lookups go to the provider you pick.

## The engine

Pages are shown by Microsoft's WebView2 runtime, which is part of Windows and is covered by the [Microsoft privacy statement](https://privacy.microsoft.com/privacystatement).

## Questions

Open an issue at <https://github.com/ladroxd/Lurt/issues>.
