# Launchpad Setup Guide (for AI Agents)

You are helping a developer set up **Launchpad**, an iOS TestFlight deployment CLI. Follow this guide to collect required information upfront and walk them through setup.

---

## Step 1: Gather Required Information

Before starting, ask the user for ALL of the following:

### Apple App Store Connect API Key (Required)
```
To deploy to TestFlight, I need your App Store Connect API credentials.

1. Go to: https://appstoreconnect.apple.com/access/integrations/api
2. Click "Generate API Key" (or use existing)
3. Select "Admin" or "App Manager" role
4. Download the .p8 file

Please provide:
- Key ID (e.g., "ABC123XYZ")
- Issuer ID (e.g., "12345678-1234-1234-1234-123456789012")
- Path where you saved the .p8 file (e.g., "~/.keys/AuthKey_ABC123XYZ.p8")
```

### Project Details (Usually Auto-detected)
```
I'll auto-detect these, but confirm:
- Is your iOS project in the `ios/` folder? (React Native/Expo default)
- What's your app's bundle identifier? (e.g., "com.company.appname")
```

---

## Step 2: Install Launchpad

Once you have the credentials, install launchpad:

```bash
# If user has Rust/Cargo
cargo install --git https://github.com/hhushhas/launchpad

# Or build from source
git clone https://github.com/hhushhas/launchpad
cd launchpad
cargo build --release
# Binary at: ./target/release/launchpad
```

---

## Step 3: Global Setup (One-time)

Run the global setup with the collected credentials:

```bash
launchpad setup
```

This will prompt for:
- **Key ID**: The API Key ID from App Store Connect
- **Issuer ID**: The Issuer ID from App Store Connect
- **Key Path**: Full path to the .p8 file

Creates: `~/.config/launchpad/config.toml`

---

## Step 4: Project Setup

In the project root directory:

```bash
launchpad init
```

This will:
1. ✓ Check/install fastlane (offers to install via brew if missing)
2. ✓ Detect iOS project path
3. ✓ Detect Xcode scheme
4. ✓ Prompt for bundle ID
5. ✓ Create `.launchpad.toml`
6. ✓ Create Fastfile if missing (with beta lanes)

---

## Step 5: Verify Setup

```bash
launchpad doctor
```

All checks should pass:
- ✓ Xcode
- ✓ fastlane
- ✓ Apple API key
- ✓ Project config
- ✓ Fastfile

---

## Step 6: Deploy!

```bash
# Build bump only
launchpad deploy

# Patch version bump (1.0.0 → 1.0.1)
launchpad deploy --patch

# Minor version bump (1.0.0 → 1.1.0)
launchpad deploy --minor
```

---

## Troubleshooting

### "No iOS project found"
- Ensure `.xcworkspace` or `.xcodeproj` exists in `ios/` folder
- For non-standard paths, use: `launchpad init --ios-path ./path/to/ios`

### "Could not detect Xcode scheme"
- Run `xcodebuild -list` in the iOS folder to see available schemes
- Use: `launchpad init --scheme YourSchemeName`

### "Apple API key not found"
- Verify the .p8 file path is correct
- Run `launchpad setup` again to reconfigure

### "Fastfile not found"
- `launchpad init` will offer to create one automatically
- Ensure the Fastfile has `beta`, `beta_patch`, and `beta_minor` lanes

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `launchpad setup` | Configure Apple API credentials (global, one-time) |
| `launchpad init` | Initialize project config + Fastfile |
| `launchpad doctor` | Verify all prerequisites |
| `launchpad deploy` | Deploy to TestFlight |
| `launchpad deploy --patch` | Bump patch version + deploy |
| `launchpad deploy --minor` | Bump minor version + deploy |

---

## Files Created

| File | Location | Purpose |
|------|----------|---------|
| `config.toml` | `~/.config/launchpad/` | Global Apple API credentials |
| `.launchpad.toml` | Project root | Project-specific config |
| `Fastfile` | `ios/fastlane/` | Fastlane deployment lanes |
