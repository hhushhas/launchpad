# Launchpad

Deploy iOS apps to TestFlight with one command.

```bash
launchpad deploy
```

---

## Prerequisites

Before using Launchpad, you need:

### 1. Xcode Command Line Tools

```bash
xcode-select --install
```

### 2. fastlane

```bash
brew install fastlane
```

### 3. App Store Connect API Key

You need an API key to upload builds without entering credentials each time.

**To create one:**

1. Go to [App Store Connect → Users and Access → Keys](https://appstoreconnect.apple.com/access/api)
2. Click the **+** button to create a new key
3. Name it something like "Launchpad" or "CI Deploy"
4. Select **Admin** or **App Manager** role
5. Click **Generate**
6. **Download the .p8 file immediately** (you can only download it once)
7. Note the **Key ID** (shown in the table, e.g., `PZAMR23N39`)
8. Note the **Issuer ID** (shown at the top of the page, e.g., `31d90bd0-7e5b-4e85-9944-3962d6234b0c`)

Store the .p8 file somewhere safe. You'll need it during setup.

---

## Setup

### Step 1: Configure API Credentials (one-time)

Run setup and follow the prompts:

```bash
launchpad setup
```

You'll be asked for:
- **API Key ID** → The key ID from App Store Connect
- **Issuer ID** → The issuer ID from App Store Connect
- **Path to .p8 file** → Where you saved the downloaded key

This creates `~/.launchpad/config.toml` and copies your key to `~/.launchpad/keys/`.

### Step 2: Initialize Your Project

In your iOS project directory:

```bash
cd /path/to/your/project
launchpad init
```

This detects your Xcode scheme and creates `.launchpad.toml`.

### Step 3: Set Up Fastfile

Your project needs a Fastfile with the required lanes. If you don't have one:

```bash
cd ios  # or wherever your .xcworkspace is
fastlane init
```

Then add these lanes to `ios/fastlane/Fastfile`:

```ruby
default_platform(:ios)

platform :ios do
  lane :beta do
    increment_build_number
    build_app(scheme: "YourAppScheme")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end

  lane :beta_patch do
    increment_version_number(bump_type: "patch")
    increment_build_number(build_number: 1)
    build_app(scheme: "YourAppScheme")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end

  lane :beta_minor do
    increment_version_number(bump_type: "minor")
    increment_build_number(build_number: 1)
    build_app(scheme: "YourAppScheme")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end
end
```

Replace `YourAppScheme` with your actual scheme name.

### Step 4: Verify Setup

```bash
launchpad doctor
```

All checks should pass:
```
✓ Xcode 16.0
✓ fastlane 2.225.0
✓ Apple API key configured
✓ Project: ./ios/MyApp.xcworkspace
✓ Fastfile found
```

---

## Usage

### Deploy (build number bump)

```bash
launchpad deploy
```

Increments build number (1.0.0 build 1 → 1.0.0 build 2).

### Deploy with version bump

```bash
launchpad deploy --patch   # 1.0.0 → 1.0.1
launchpad deploy --minor   # 1.0.0 → 1.1.0
```

### Skip git checks

```bash
launchpad deploy --skip-git-check
```

### Skip git tagging

```bash
launchpad deploy --no-tag
```

---

## Project Config

The `.launchpad.toml` file in your project root:

```toml
[project]
ios_path = "ios"              # Path to .xcworkspace
scheme = "MyApp"              # Xcode scheme
bundle_id = "com.you.myapp"   # Bundle identifier

[deploy]
git_tag = true                # Create git tags (v1.0.0)
push_tags = true              # Push tags to remote
clean_artifacts = true        # Remove IPA after upload
```

---

## Troubleshooting

### "Apple API key not configured"

Run `launchpad setup` to configure your credentials.

### "Fastfile not found"

Run `fastlane init` in your iOS directory, then add the required lanes.

### "No iOS project found"

Make sure you're in a directory with a `.xcworkspace` or `.xcodeproj` file, or use `--ios-path` to specify the location.

### Build fails with signing errors

Make sure your Fastfile includes proper code signing. Add to your lane:

```ruby
setup_ci if ENV['CI']
match(type: "appstore", readonly: true)  # if using match
```

### "Git working directory is not clean"

Commit or stash your changes first, or use `--skip-git-check`.

---

## AI-Assisted Setup

Using an AI coding assistant (Claude Code, Cursor, Copilot, etc.)? Copy the contents of [`AGENT_PROMPT.md`](./AGENT_PROMPT.md) into your project's AI instructions file (e.g., `CLAUDE.md`, `.cursorrules`).

The AI will:
1. Ask for your App Store Connect credentials upfront
2. Guide you through each setup step
3. Troubleshoot common issues

This is the fastest way to get started - no docs reading required!
