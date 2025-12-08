class Cco < Formula
  desc "Claude Code Orchestrator - AI-powered development automation platform"
  homepage "https://github.com/visiquate/cco"
  license "MIT"
  version "2025.12.28"

  on_macos do
    on_arm do
      url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-aarch64-apple-darwin.tar.gz"
      sha256 "09d8a22cde5fe1d3e268cd0fdf5d08db797f5540602252ee857466c23614500b"
    end
    on_intel do
      odie "x86_64 macOS artifact not published for v#{version}"
    end
  end

  on_linux do
    on_arm do
      odie "Linux aarch64 artifact not published for v#{version}"
    end
    on_intel do
      url "https://github.com/visiquate/cco/releases/download/v#{version}/cco-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "730dfe0888d24a403a86736d15f18ffc0bab2e6341bd6b38b79b7b5602f1022a"
    end
  end

  def install
    bin.install "cco"
  end

  test do
    assert_match("cco", shell_output("#{bin}/cco --version"))
  end
end
