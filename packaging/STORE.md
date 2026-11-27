# Publishing Lurt in the Microsoft Store

The Store signs MSIX packages itself, so Store installs show no SmartScreen warning. It does not sign the `lurt.exe` on GitHub.

1. Register a developer account at <https://storedeveloper.microsoft.com>.
2. In Partner Center, create a new MSIX app and reserve its name.
3. Open **Product identity** and copy `Package/Identity/Name`, `Package/Identity/Publisher` and `PublisherDisplayName`.
4. In the GitHub repository, add them under **Settings → Secrets and variables → Actions → Variables** as `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER` and `MSIX_PUBLISHER_NAME` (add `MSIX_DISPLAY_NAME` if the reserved name isn't "Lurt"). The release workflow then attaches the package to its run as the `lurt-msix` artifact. Or pack it yourself with `msix.ps1`.
5. Upload the `.msix` in a new submission. The `runFullTrust` capability needs a short reason: "Lurt is a Win32 desktop web browser using WebView2." Link [PRIVACY.md](../PRIVACY.md) as the privacy policy.

Check before the first submission, with the installed package: starting, sign-ins surviving a restart, importing from other browsers, downloads and opening them. Windows redirects a packaged app's new AppData files to the package's own folder, so a Store install starts with its own profile.
