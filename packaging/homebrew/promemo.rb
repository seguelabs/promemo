class Promemo < Formula
  desc "Git-native project memory for AI-assisted development"
  homepage "https://github.com/seguelabs/promemo"
  url "https://github.com/seguelabs/promemo/archive/refs/tags/v0.4.0.tar.gz"
  sha256 "REPLACE_WITH_V0_4_0_SOURCE_TARBALL_SHA"
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
