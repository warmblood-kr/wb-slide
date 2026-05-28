#!/bin/sh
set -e

REPO="warmblood-kr/wb-slide"
INSTALL_DIR="${WB_SLIDE_INSTALL_DIR:-/usr/local/bin}"
BINARY="wb-slide"

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

main() {
  VERSION="${1:-$(get_latest_version)}"
  if [ -z "$VERSION" ]; then
    echo "Error: could not determine latest version"
    exit 1
  fi

  PLATFORM="$(detect_platform)"
  echo "Installing wb-slide ${VERSION} for ${PLATFORM}..."

  case "$PLATFORM" in
    *windows*)
      ASSET="wb-slide-${PLATFORM}.zip"
      URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
      TMPDIR="$(mktemp -d)"
      curl -fsSL "$URL" -o "${TMPDIR}/${ASSET}"
      unzip -o "${TMPDIR}/${ASSET}" -d "${TMPDIR}" > /dev/null
      mv "${TMPDIR}/${BINARY}.exe" "${INSTALL_DIR}/${BINARY}.exe"
      rm -rf "$TMPDIR"
      echo "Installed to ${INSTALL_DIR}/${BINARY}.exe"
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
        sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
      fi
      rm -rf "$TMPDIR"
      echo "Installed to ${INSTALL_DIR}/${BINARY}"
      ;;
  esac

  echo ""
  echo "Run 'wb-slide show' in a directory with slides.md to start presenting."
}

main "$@"
