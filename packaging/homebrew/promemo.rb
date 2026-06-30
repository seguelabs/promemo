class Promemo < Formula
  desc "Git-native project memory for AI-assisted development"
  homepage "https://github.com/seguelabs/promemo"
  url "https://github.com/seguelabs/promemo/archive/refs/tags/v0.4.0.tar.gz"
  sha256 "989b4f4cdc60436f7d230ae2b2e48e1c46c23716fc307f131d51aed1169dbfb0"
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
