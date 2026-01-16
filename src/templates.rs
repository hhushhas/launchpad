/// Fastfile template with placeholder for scheme name
pub const FASTFILE_TEMPLATE: &str = r#"default_platform(:ios)

platform :ios do
  lane :beta do
    increment_build_number
    build_app(scheme: "{{SCHEME}}")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end

  lane :beta_patch do
    increment_version_number(bump_type: "patch")
    increment_build_number(build_number: 1)
    build_app(scheme: "{{SCHEME}}")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end

  lane :beta_minor do
    increment_version_number(bump_type: "minor")
    increment_build_number(build_number: 1)
    build_app(scheme: "{{SCHEME}}")
    upload_to_testflight(
      api_key_path: ENV["APP_STORE_CONNECT_API_KEY_KEY_FILEPATH"],
      skip_waiting_for_build_processing: true
    )
  end
end
"#;

/// Generate a Fastfile with the scheme name filled in
pub fn generate_fastfile(scheme: &str) -> String {
    FASTFILE_TEMPLATE.replace("{{SCHEME}}", scheme)
}

/// Example .launchpad.toml for team reference
pub const LAUNCHPAD_TOML_EXAMPLE: &str = r#"# Launchpad configuration file
# Copy this to .launchpad.toml and customize for your project

[project]
ios_path = "ios"           # Path to iOS project directory
scheme = "YourAppScheme"   # Xcode scheme name
bundle_id = "com.example.app"

[deploy]
git_tag = true             # Create git tags after deploy
push_tags = true           # Push tags to remote
clean_artifacts = true     # Clean build artifacts after deploy
"#;
