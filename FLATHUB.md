# Publishing GTLLM to Flathub

This guide explains how to publish GTLLM to Flathub, the Linux app store.

## Prerequisites

1. **Flatpak and Flatpak Builder** installed:
   ```bash
   # On Fedora/RHEL
   sudo dnf install flatpak flatpak-builder
   
   # On Ubuntu/Debian
   sudo apt install flatpak flatpak-builder
   ```

2. **Flathub repository** added:
   ```bash
   flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
   ```

3. **Rust SDK extension** installed:
   ```bash
   flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//24.08
   ```

4. **GitHub account** and repository set up (if not already)

## Step 1: Create Application Icon

Before submitting, you need to create an application icon:

1. Create a scalable SVG icon: `com.chironjit.gtllm.svg` (recommended)
   - Should be at least 512x512 pixels when rendered
   - Place it in the project root

2. Or create a PNG icon: `com.chironjit.gtllm.png`
   - Should be 512x512 pixels
   - Place it in the project root

The manifest will automatically install the icon if it exists.

## Step 2: Test the Flatpak Build Locally

1. **Build the Flatpak locally:**
   ```bash
   flatpak-builder --user --install build-dir com.chironjit.gtllm.yml --force-clean
   ```

2. **Run the application:**
   ```bash
   flatpak run com.chironjit.gtllm
   ```

3. **Test all features** to ensure everything works correctly in the sandboxed environment.

## Step 3: Fix the Manifest

The current manifest has placeholder values that need to be fixed:

1. **Cargo vendor sources**: The `cargo-vendor-x86_64` and `cargo-vendor-aarch64` modules have placeholder URLs and SHA256 values. You have two options:

   **Option A: Use Flathub's cargo-vendor (Recommended)**
   - Remove the `cargo-vendor` module
   - Update the vendor modules to use Flathub's cargo-vendor archive
   - Get the correct SHA256 from Flathub's repository

   **Option B: Generate your own vendor archive**
   - Run `cargo vendor` locally
   - Create a tar.gz archive
   - Host it somewhere (GitHub releases, etc.)
   - Update the manifest with the correct URL and SHA256

2. **Update version numbers** in `com.chironjit.gtllm.appdata.xml` to match your actual release version.

## Step 4: Create Flathub Repository

1. **Fork the Flathub repository:**
   - Go to https://github.com/flathub/flathub
   - Fork it to your GitHub account

2. **Clone your fork:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/flathub.git
   cd flathub
   ```

3. **Create your application directory:**
   ```bash
   mkdir -p apps/com.chironjit.gtllm
   ```

4. **Copy the manifest and related files:**
   ```bash
   cp /path/to/gtllm/com.chironjit.gtllm.yml apps/com.chironjit.gtllm/
   cp /path/to/gtllm/com.chironjit.gtllm.desktop apps/com.chironjit.gtllm/
   cp /path/to/gtllm/com.chironjit.gtllm.appdata.xml apps/com.chironjit.gtllm/
   # If you have an icon
   cp /path/to/gtllm/com.chironjit.gtllm.svg apps/com.chironjit.gtllm/  # or .png
   ```

5. **Update the manifest** to use the correct source location:
   - Change the `sources` section to point to your GitHub repository
   - Example:
     ```yaml
     sources:
       - type: git
         url: https://github.com/chironjit/gtllm.git
         tag: v0.1.0
         commit: abc123...
     ```

## Step 5: Validate the Manifest

1. **Install flatpak-builder-lint:**
   ```bash
   # On Fedora
   sudo dnf install flatpak-builder-lint
   
   # Or build from source
   git clone https://github.com/flathub/flatpak-builder-lint
   cd flatpak-builder-lint
   make
   sudo make install
   ```

2. **Lint your manifest:**
   ```bash
   flatpak-builder-lint com.chironjit.gtllm.yml
   ```

3. **Fix any issues** reported by the linter.

## Step 6: Submit to Flathub

1. **Commit and push your changes:**
   ```bash
   cd flathub
   git add apps/com.chironjit.gtllm/
   git commit -m "Add GTLLM application"
   git push origin main
   ```

2. **Create a Pull Request:**
   - Go to https://github.com/flathub/flathub
   - Click "New Pull Request"
   - Select your fork and branch
   - Fill out the PR template
   - Submit the PR

3. **Wait for review:**
   - Flathub maintainers will review your submission
   - They may request changes or ask questions
   - Be responsive to feedback

## Step 7: After Approval

Once your PR is merged:

1. **Your app will be built automatically** by Flathub's build system
2. **It will appear on Flathub** after the build completes successfully
3. **Users can install it** with:
   ```bash
   flatpak install flathub com.chironjit.gtllm
   ```

## Updating Your App

To update your app on Flathub:

1. **Update the manifest** in your Flathub fork with the new version
2. **Update the source tag/commit** to point to the new release
3. **Update appdata.xml** with the new release information
4. **Create a new PR** with the changes

## Important Notes

- **Sandboxing**: Your app runs in a sandbox. Make sure all file paths use XDG directories (which your app already does with the `dirs` crate)
- **Network access**: The manifest includes `--share=network` so your app can access OpenRouter APIs
- **File access**: The manifest includes `--filesystem=home` and `--filesystem=xdg-config` for settings storage
- **Icons**: Make sure to create and include an application icon
- **Versioning**: Keep version numbers in sync between Cargo.toml, appdata.xml, and git tags

## Resources

- [Flathub Documentation](https://docs.flathub.org/)
- [Flatpak Builder Documentation](https://docs.flatpak.org/en/latest/flatpak-builder.html)
- [AppData Specification](https://www.freedesktop.org/software/appstream/docs/chap-Quickstart.html)
- [Flathub GitHub Repository](https://github.com/flathub/flathub)

## Troubleshooting

### Build fails with "cargo: command not found"
- Make sure you installed the Rust SDK extension: `flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//24.08`

### App doesn't have network access
- Check that `--share=network` is in the `finish-args` section

### App can't save settings
- Check that `--filesystem=xdg-config` and `--filesystem=home` are in `finish-args`
- Verify your app uses XDG directories (which it does via the `dirs` crate)

### Icon not showing
- Make sure the icon file exists and is named correctly
- Check that the `post-install` script installs the icon to the correct location
- Verify the icon format (SVG or PNG) matches what you created
