#!/bin/sh
set -e

REPO="warmblood-kr/wb-slide"
BINARY="wb-slide"

# Resolve install dir.
# Priority:
#   1. WB_SLIDE_INSTALL_DIR (explicit override)
#   2. ~/.local/bin if it's on PATH or we'll mention it
#   3. /usr/local/bin (system-wide, may need sudo)
resolve_install_dir() {
  if [ -n "$WB_SLIDE_INSTALL_DIR" ]; then
    echo "$WB_SLIDE_INSTALL_DIR"
    return
  fi

  USER_BIN="${HOME}/.local/bin"
  if [ -d "$USER_BIN" ] && [ -w "$USER_BIN" ]; then
    echo "$USER_BIN"
    return
  fi

  # Prefer user-owned location even if we have to create it,
  # to avoid sudo on macOS/Linux.
  if [ -w "$HOME" ]; then
    echo "$USER_BIN"
    return
  fi

  echo "/usr/local/bin"
}

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//'
}

detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$OS" in
    Darwin)
      case "$ARCH" in
        arm64|aarch64) echo "macos-arm64" ;;
        *) echo "unsupported: macOS $ARCH (only Apple Silicon supported)" && exit 1 ;;
      esac
      ;;
    Linux)
      case "$ARCH" in
        x86_64|amd64) echo "linux-x64" ;;
        *) echo "unsupported: Linux $ARCH" && exit 1 ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*)
      echo "windows-x64"
      ;;
    *)
      echo "unsupported: $OS" && exit 1
      ;;
  esac
}

in_path() {
  case ":$PATH:" in
    *":$1:"*) return 0 ;;
    *) return 1 ;;
  esac
}

main() {
  VERSION="${1:-$(get_latest_version)}"
  if [ -z "$VERSION" ]; then
    echo "Error: could not determine latest version"
    exit 1
  fi

  PLATFORM="$(detect_platform)"
  INSTALL_DIR="$(resolve_install_dir)"

  echo "Installing wb-slide ${VERSION} for ${PLATFORM}..."
  echo "Target: ${INSTALL_DIR}"
  echo ""

  mkdir -p "$INSTALL_DIR" 2>/dev/null || true

  case "$PLATFORM" in
    *windows*)
      ASSET="wb-slide-${PLATFORM}.zip"
      URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
      TMPDIR="$(mktemp -d)"
      curl -fsSL "$URL" -o "${TMPDIR}/${ASSET}"
      unzip -o "${TMPDIR}/${ASSET}" -d "${TMPDIR}" > /dev/null
      mv "${TMPDIR}/${BINARY}.exe" "${INSTALL_DIR}/${BINARY}.exe"
      rm -rf "$TMPDIR"
      echo "Installed: ${INSTALL_DIR}/${BINARY}.exe"
      ;;
    *)
      ASSET="wb-slide-${PLATFORM}.tar.gz"
      URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
      TMPDIR="$(mktemp -d)"
      curl -fsSL "$URL" -o "${TMPDIR}/${ASSET}"
      tar xzf "${TMPDIR}/${ASSET}" -C "${TMPDIR}"
      chmod +x "${TMPDIR}/${BINARY}"

      if [ -w "$INSTALL_DIR" ]; then
        mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
      else
        echo "${INSTALL_DIR} is not writable, falling back to sudo."
        sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
      fi
      rm -rf "$TMPDIR"
      echo "Installed: ${INSTALL_DIR}/${BINARY}"
      ;;
  esac

  echo ""

  # Path check
  if ! in_path "$INSTALL_DIR"; then
    echo "WARNING: ${INSTALL_DIR} is not in your PATH."
    echo "Add this to your shell config (~/.zshrc, ~/.bashrc, etc.):"
    echo ""
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
  fi

  echo "Run 'wb-slide show' in a directory with slides.md to start presenting."
}

main "$@"
