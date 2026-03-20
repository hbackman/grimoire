# Grimoire — Distribution Plan

## Overview

Grimoire is a Tauri v2 desktop application (Rust backend + Vue 3 frontend).
Tauri's built-in tooling makes cross-platform distribution straightforward.

---

## 1. Building the App

### Development build
```bash
cd src-tauri
cargo build
# In project root:
npm run tauri dev
```

### Production bundle
```bash
npm run tauri build
```

This produces platform-native bundles in `src-tauri/target/release/bundle/`:

| Platform | Output |
|----------|--------|
| macOS    | `.app` bundle + `.dmg` installer |
| Windows  | `.exe` installer (NSIS or WiX) + `.msi` |
| Linux    | `.deb`, `.rpm`, `.AppImage` |

Tauri's bundler is configured in `src-tauri/tauri.conf.json` under the
`bundle` key. Key settings: `identifier`, `icon`, `category`, `shortDescription`.

---

## 2. Distribution Options

### Option A: GitHub Releases (recommended)
- Push a git tag (e.g. `v1.0.0`)
- GitHub Actions CI builds for all three platforms in parallel
- Artifacts are uploaded to a GitHub Release automatically
- Users download directly from the Releases page
- Free, no infrastructure needed
- Integrates naturally with Tauri's updater plugin (see §4)

### Option B: Dedicated website
- Host a landing page (GitHub Pages / Vercel / Netlify)
- Link to the GitHub Release binaries
- Add install instructions per platform
- Optionally bundle a homebrew tap for macOS: `brew install --cask grimoire`

### Option C: Package managers
- **macOS:** Homebrew cask (`homebrew-cask` or a custom tap)
- **Windows:** winget manifest (`winget install Grimoire`)
- **Linux:** AUR (Arch), `.deb` in a PPA, Flatpak on Flathub
- These add discoverability but require ongoing maintenance

### Option D: Direct download page
- Host the binaries yourself (S3, Cloudflare R2, etc.)
- Gives full control of analytics and versioning
- More effort; not recommended until significant user base

**Recommended path:** GitHub Releases for v1, add a landing page and
Homebrew tap later when there's traction.

---

## 3. Code Signing

### macOS
1. Enroll in the **Apple Developer Program** ($99/year)
2. Create a **Developer ID Application** certificate in Xcode / Apple Portal
3. Export the `.p12` certificate
4. In GitHub Actions, store the base64-encoded cert and password as secrets
5. In `tauri.conf.json` set `bundle.macOS.signingIdentity`
6. After signing, **notarize** via `xcrun notarytool` (required since macOS 10.15)
7. Tauri v2 supports `notarize: true` in the bundle config — it calls the
   Apple notarization API automatically when `APPLE_API_KEY` etc. are set

Without signing/notarization, macOS will show a Gatekeeper warning and many
users won't be able to open the app without right-clicking → Open.

### Windows
1. Obtain a **Code Signing Certificate** from a CA (DigiCert, Sectigo, etc.)
   - Standard OV cert: ~$200-400/year
   - EV cert (~$500+/year) avoids SmartScreen warnings immediately
2. In GitHub Actions, store the PFX file as a base64 secret
3. In `tauri.conf.json` set `bundle.windows.certificateThumbprint` or use
   the `TAURI_PRIVATE_KEY` / `TAURI_KEY_PASSWORD` env vars
4. Tauri will call `signtool.exe` during the build

Without signing, Windows SmartScreen will show "Unknown publisher" warnings
and may block the installer entirely for EV-less certs until reputation builds.

### Linux
- Linux packages are generally not signed at the binary level
- Debian/RPM repos can be GPG-signed (`.deb` repos use `Release.gpg`)
- AppImage supports optional code signing
- For most use cases, Linux signing is optional for initial distribution

---

## 4. Auto-Update via Tauri's Updater Plugin

Tauri v2 ships a first-class updater (`tauri-plugin-updater`).

### Setup
1. Add to `Cargo.toml`:
   ```toml
   tauri-plugin-updater = "2"
   ```
2. Add to `tauri.conf.json`:
   ```json
   {
     "plugins": {
       "updater": {
         "pubkey": "YOUR_PUBLIC_KEY_HERE",
         "endpoints": [
           "https://github.com/hbackman/grimoire/releases/latest/download/latest.json"
         ]
       }
     }
   }
   ```
3. Generate signing keys:
   ```bash
   npm run tauri signer generate -- -w ~/.tauri/grimoire.key
   ```
4. The CI pipeline signs each release artifact and produces a `latest.json`
   manifest (signature + URL + version) uploaded alongside the binaries.

### Update flow
- On launch, the app checks the `endpoints` URL
- If a newer version exists, it downloads and verifies the signature
- The user is prompted to install; on next launch the update is applied
- The updater can also run in the background silently

### Update manifest format
```json
{
  "version": "1.1.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-01-01T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/.../grimoire_1.1.0_aarch64.dmg"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../grimoire_1.1.0_x64-setup.exe"
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../grimoire_1.1.0_amd64.AppImage.tar.gz"
    }
  }
}
```

---

## 5. Suggested GitHub Actions CI/CD Pipeline

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'   # triggers on v1.0.0, v1.2.3, etc.

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          # macOS Universal (Intel + Apple Silicon)
          - platform: macos-latest
            args: --target universal-apple-darwin

          # Windows x64
          - platform: windows-latest
            args: ''

          # Linux x64
          - platform: ubuntu-22.04
            args: ''

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: >-
            ${{ matrix.platform == 'macos-latest' &&
              'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      # Linux only: install GTK/WebKit system deps
      - name: Install Linux deps
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf

      - name: Install frontend deps
        run: npm ci

      # Run pure-logic tests (no GTK required)
      - name: Run Rust unit tests
        run: |
          cd src-tauri
          cargo test -p addon-core

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # macOS code signing
          APPLE_CERTIFICATE:          ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY:     ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID:                   ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD:             ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID:              ${{ secrets.APPLE_TEAM_ID }}
          # Windows code signing
          WINDOWS_CERTIFICATE:         ${{ secrets.WINDOWS_CERTIFICATE }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
          # Tauri updater signing
          TAURI_SIGNING_PRIVATE_KEY:          ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName:       ${{ github.ref_name }}
          releaseName:   'Grimoire ${{ github.ref_name }}'
          releaseBody:   'See CHANGELOG.md for details.'
          releaseDraft:  false
          prerelease:    false
          args:          ${{ matrix.args }}
```

### What `tauri-action` does
- Installs Tauri CLI
- Runs `npm run tauri build` with your `args`
- Signs and notarizes on macOS (if credentials are provided)
- Signs on Windows (if credentials are provided)
- Uploads all bundle artifacts to the GitHub Release
- Produces and uploads `latest.json` for the updater

### Secrets to configure
| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Jane Doe (TEAM123)` |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | App-specific password for notarization |
| `APPLE_TEAM_ID` | 10-char team ID from Apple Portal |
| `WINDOWS_CERTIFICATE` | Base64-encoded `.pfx` |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the `.pfx` |
| `TAURI_SIGNING_PRIVATE_KEY` | Generated by `tauri signer generate` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the signing key |

---

## 6. Release Checklist

Before tagging a release:
- [ ] Bump version in `src-tauri/Cargo.toml` and `package.json`
- [ ] Update `CHANGELOG.md`
- [ ] Run `cargo test -p addon-core` locally
- [ ] Test production build on at least one platform
- [ ] Tag: `git tag v1.x.x && git push --tags`
- [ ] Verify GitHub Actions build succeeds
- [ ] Verify release artifacts appear on the GitHub Releases page
- [ ] Verify auto-update works from the previous version
