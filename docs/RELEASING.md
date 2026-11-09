# Releasing

Releases are cut from `main` via a version-bump PR and a tag. Replace `0.4.0` below with the new version
(pre-1.0: `0.x.0` for features, `0.x.y` for fixes only).

1. **Prepare.** Make sure everything for the release is merged and CI on `main` is green:
   ```sh
   git switch main && git pull --ff-only
   git log "$(git describe --tags --abbrev=0)"..HEAD --oneline   # what goes into the release
   ```
2. **Branch.**
   ```sh
   git switch -c release/v0.4.0
   ```
3. **Bump the version in all three places** (the `Release` workflow verifies they match the tag):
   `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`.
   Then refresh the lockfiles:
   ```sh
   npm install --package-lock-only
   (cd src-tauri && cargo update -p jot)
   ```
   `git diff` must show only the version lines (5 files: the three above plus `package-lock.json`, `src-tauri/Cargo.lock`).
   Optionally update the version placeholder in `.github/ISSUE_TEMPLATE/bug_report.yml`.
4. **Commit and PR.**
   ```sh
   git commit -am "chore: release v0.4.0"
   git push -u origin release/v0.4.0
   gh pr create --title "chore: release v0.4.0" --body "Version bump to 0.4.0"
   ```
   Wait for CI to pass, then merge the PR.
5. **Tag the merge commit on `main` and push the tag:**
   ```sh
   git switch main && git pull --ff-only
   git log -1 --oneline        # must be the release PR's merge commit
   git tag v0.4.0
   git push origin v0.4.0
   git branch -d release/v0.4.0
   ```
6. **The `Release` workflow starts** from the tag. It verifies the tag is on `main` and that the versions
   match, builds macOS (arm64 + x64), Linux (deb/rpm/AppImage), Windows (NSIS/MSI) and Android (APK/AAB),
   and attaches everything to a **draft** release with auto-generated notes.
7. **Approve.** Publishing needs a manual approval (GitHub Environment `release`, required reviewer:
   maintainer). Check the draft's assets and notes (edit the notes if needed), then approve the pending
   `Release veröffentlichen` job.
8. **Verify.** The release is public, all platform artifacts are attached, and the Android APK/AAB is signed.

If the workflow fails or the tag is wrong, delete the draft release and the tag
(`git push origin :refs/tags/v0.4.0 && git tag -d v0.4.0`), fix, and re-tag.
A tag that points to a published release must not be moved; publish a new patch version instead.

## Secrets (repository settings → Secrets and variables → Actions)

### Android (required for installable APK/AAB)

| Secret                      | Content                      |
| --------------------------- | ---------------------------- |
| `ANDROID_KEYSTORE_BASE64`   | `base64 -i release.keystore` |
| `ANDROID_KEYSTORE_PASSWORD` | keystore password            |
| `ANDROID_KEY_ALIAS`         | key alias                    |
| `ANDROID_KEY_PASSWORD`      | key password                 |

Create a keystore once:
`keytool -genkey -v -keystore release.keystore -alias jot -keyalg RSA -keysize 2048 -validity 10000`.
**Back it up** – losing it means no more updates for existing installs.
Without these secrets the Android artifacts are unsigned.

### macOS

Builds are unsigned (no Apple Developer account); Gatekeeper warns on first launch.
The workflow deliberately does not pass any `APPLE_*` variables: even empty values make
Tauri try to import a signing certificate and fail the build. To enable signing later, add
`APPLE_CERTIFICATE` (base64 of the `.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` as secrets **and** re-add them to the
`tauri-action` step's `env` in `release.yml`.

### Windows

Builds are unsigned (SmartScreen warns) unless a code-signing certificate is configured.
