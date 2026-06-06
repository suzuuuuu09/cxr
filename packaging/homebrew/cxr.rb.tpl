class Cxr < Formula
  desc "A tool to generate a directory structure from a YAML template"
  homepage "https://github.com/suzuuuuu09/cxr"
  url "https://github.com/suzuuuuu09/cxr/releases/download/v{{VERSION}}/cxr-{{VERSION}}.tar.gz"
  sha256 "{{SHA256}}"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cxr --version")
  end
end
