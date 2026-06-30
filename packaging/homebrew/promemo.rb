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
