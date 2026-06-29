# Distribution

Promemo distribution should start with the simplest reliable path and then add
package managers once release artifacts are stable.

## Version Source

The CLI version is the Cargo package version in `Cargo.toml`:

```toml
version = "0.3.8"
```

The CLI exposes that value with:

```bash
promemo --version
```

Release tags should match the Cargo version with a leading `v`:

```txt
Cargo.toml: 0.3.8
Git tag:    v0.3.8
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
   git tag v0.3.8
   git push origin v0.3.8
   ```

7. Confirm the `Release` workflow completes and attaches platform archives to
   the GitHub Release.

## Cargo From Git

This is the first supported distribution path because it needs no external
registry setup:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.8
```

Update to a newer tag:

```bash
cargo install --git https://github.com/seguelabsai/promemo.git --tag v0.3.8 --force
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

## npm

npm is the first package-manager distribution target for Promemo because it gives
AI-tool users a quick cross-platform path to `npx promemo` and global installs.

Expected user flow:

```bash
npx promemo --version
npx promemo init
npm install -g promemo
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

Homebrew can follow after npm for users who prefer native package managers.

Expected user flow:

```bash
brew install seguelabsai/tap/promemo
brew upgrade promemo
```

Implementation path:

1. Create a Homebrew tap repository such as `homebrew-tap`.
2. Add a `Formula/promemo.rb` formula.
3. Point the formula at the GitHub release archive for the current tag.
4. Update the formula SHA for each release.

Formula template:

```ruby
class Promemo < Formula
  desc "Git-native project memory for AI-assisted development"
  homepage "https://github.com/seguelabsai/promemo"
  url "https://github.com/seguelabsai/promemo/archive/refs/tags/v0.3.8.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "promemo", shell_output("#{bin}/promemo --version")
  end
end
```

Prebuilt bottles can come later.
