# Distribution

Promemo distribution supports npm, GitHub release binaries, source installs, a
direct install script, and a Homebrew formula seed.

## Version Source

The CLI version is the Cargo package version in `Cargo.toml`:

```toml
version = "0.3.9"
```

The CLI exposes that value with:

```bash
promemo --version
```

Release tags should match the Cargo version with a leading `v`:

```txt
Cargo.toml: 0.3.9
Git tag:    v0.3.9
```

## Release Flow

1. Merge feature work into `develop`.
2. Create a release branch from `develop`.
3. Update `Cargo.toml` to the release version.
4. Run validation:

   ```bash
   cargo fmt
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   cargo run -- --version
   ```

5. Merge the release branch.
6. Tag the release commit:

   ```bash
   git tag v0.3.9
   git push origin v0.3.9
   ```

7. Confirm the `Release` workflow completes and attaches platform archives to
   the GitHub Release.

## Cargo From Git

This is the first supported distribution path because it needs no external
registry setup:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.9
```

Update to a newer tag:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.9 --force
```

## GitHub Releases

GitHub Releases publish platform binaries from `.github/workflows/release.yml`
when a `v*` tag is pushed. The workflow can also be run manually for validation,
but it only uploads release assets for tag builds.

Initial target artifacts:

```txt
promemo-aarch64-apple-darwin.tar.gz
promemo-x86_64-unknown-linux-gnu.tar.gz
promemo-x86_64-pc-windows-msvc.zip
SHA256SUMS
```

Those artifacts become the source for npm and Homebrew wrappers.

## Direct Binary Install Script

Users can install the latest release binary without cloning the repository:

```bash
curl -fsSL https://raw.githubusercontent.com/seguelabsai/promemo/main/scripts/install.sh | sh
```

The script:

- detects macOS arm64, Linux x64, or Windows x64 shells
- downloads the matching release archive
- downloads `SHA256SUMS`
- verifies the archive when `sha256sum` or `shasum` is available
- installs to `$HOME/.local/bin` by default

Override the install location or version:

```bash
PROMEMO_INSTALL_DIR=/usr/local/bin PROMEMO_VERSION=v0.3.9 sh scripts/install.sh
```

## npm

npm is the first package-manager distribution target for Promemo because it gives
AI-tool users a quick cross-platform path to `npx promemo` and global installs.

Expected user flow:

```bash
npx -y promemo@latest --version
npx -y promemo@latest init
npm install -g promemo@latest
promemo --version
npm update -g promemo
```

Implementation path:

1. Publish GitHub release binaries.
2. Keep `npm/promemo/package.json` in sync with the Cargo version.
3. Run `npm run prepare-binaries` from `npm/promemo` before publishing.
4. The prepare script downloads all GitHub release artifacts, verifies them
   against `SHA256SUMS`, and places the native binaries under `vendor/`.
5. `npm publish` bundles those binaries directly in the npm package.
6. The `promemo` npm bin selects the matching bundled binary for the user's
   platform.

Users do not need GitHub authentication when installing from npm.

Publish from the npm package directory:

```bash
cd npm/promemo
npm login
npm run prepare-binaries
npm publish
```

For scoped packages use `npm publish --access public`, but the unscoped
`promemo` package does not need that flag.

## Homebrew

Homebrew support starts from the formula seed in
`packaging/homebrew/promemo.rb`. Copy that file into a tap repository at
`Formula/promemo.rb`.

Expected user flow:

```bash
brew install seguelabsai/tap/promemo
brew upgrade promemo
```

Tap setup:

1. Create a Homebrew tap repository such as `homebrew-tap`.
2. Copy `packaging/homebrew/promemo.rb` to `Formula/promemo.rb`.
3. Update the formula URL and SHA for each release.
4. Push the tap.

Current formula seed:

```ruby
class Promemo < Formula
  desc "Git-native project memory for AI-assisted development"
  homepage "https://github.com/seguelabsai/promemo"
  url "https://github.com/seguelabsai/promemo/archive/refs/tags/v0.3.9.tar.gz"
  sha256 "78d08a7701b784903e9a48c646bf5832b9b5dc554f47af1f8e5f4e8ee9a6251d"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "promemo", shell_output("#{bin}/promemo --version")
    system bin/"promemo", "--help"
  end
end
```

Prebuilt bottles can come later.
