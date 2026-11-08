# Releasing

1. Bump the version in **all three** places (CI verifies they match the tag):
   `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`
   (and run `cargo update -p jot` / `npm install --package-lock-only` to refresh lockfiles).
2. Open a PR, merge it into `main`.
3. Tag the merge commit on `main` and push the tag:
   ```sh
   git checkout main && git pull
   git tag v0.4.0
   git push origin v0.4.0
   ```
4. The `Release` workflow verifies the tag is on `main`, builds macOS (arm64 + x64),
   Linux (deb/rpm/AppImage), Windows (NSIS/MSI) and Android (APK/AAB), attaches
   everything to a draft release. Publishing needs a manual approval
   (GitHub Environment `release`, required reviewer: maintainer): check the draft's
   assets, then approve the pending `Release veröffentlichen` job.

## Secrets (repository settings → Secrets and variables → Actions)

### Android (required for installable APK/AAB)
| Secret | Content |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | `base64 -i release.keystore` |
| `ANDROID_KEYSTORE_PASSWORD` | keystore password |
| `ANDROID_KEY_ALIAS` | key alias |
| `ANDROID_KEY_PASSWORD` | key password |

Create a keystore once:
`keytool -genkey -v -keystore release.keystore -alias jot -keyalg RSA -keysize 2048 -validity 10000`.
**Back it up** – losing it means no more updates for existing installs.
Without these secrets the Android artifacts are unsigned.

### macOS (optional; without them the app is unsigned and Gatekeeper warns)
`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`.

### Windows
Builds are unsigned (SmartScreen warns) unless a code-signing certificate is configured.
