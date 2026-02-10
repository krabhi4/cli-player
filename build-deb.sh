#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────
#  Build a .deb package for CLI Music Player
#  Usage:  ./build-deb.sh
#  Output: cli-music-player_<version>_<arch>.deb
# ─────────────────────────────────────────────────────────
set -euo pipefail

APP_NAME="cli-music-player"
VERSION="2.0.2"
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
INSTALL_DIR="/opt/${APP_NAME}"
PKG_DIR="$(mktemp -d)"
SRC_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "🔨 Building ${APP_NAME} v${VERSION} (${ARCH})..."

# ─── Create directory structure ────────────────────────
mkdir -p "${PKG_DIR}${INSTALL_DIR}"
mkdir -p "${PKG_DIR}/usr/local/bin"
mkdir -p "${PKG_DIR}/DEBIAN"

# ─── Copy application source (excluding __pycache__) ──
cp -r "${SRC_DIR}/src" "${PKG_DIR}${INSTALL_DIR}/"
find "${PKG_DIR}" -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
cp "${SRC_DIR}/pyproject.toml" "${PKG_DIR}${INSTALL_DIR}/"
cp "${SRC_DIR}/README.md" "${PKG_DIR}${INSTALL_DIR}/"

# ─── Create launcher script ───────────────────────────
cat > "${PKG_DIR}/usr/local/bin/music-player" << 'LAUNCHER'
#!/usr/bin/env bash
# CLI Music Player launcher
INSTALL_DIR="/opt/cli-music-player"
VENV_DIR="${INSTALL_DIR}/venv"

if [ ! -d "${VENV_DIR}" ]; then
    echo "Error: Virtual environment not found. Run: sudo music-player --setup"
    exit 1
fi

if [ "$1" = "--setup" ]; then
    echo "Setting up CLI Music Player..."
    python3 -m venv "${VENV_DIR}"
    source "${VENV_DIR}/bin/activate"
    pip install --upgrade pip > /dev/null 2>&1
    pip install --force-reinstall --no-cache-dir "${INSTALL_DIR}" > /dev/null 2>&1
    echo "✓ Setup complete! Run 'music-player' to start."
    exit 0
fi

source "${VENV_DIR}/bin/activate"
exec python3 -m cli_music_player "$@"
LAUNCHER
chmod 755 "${PKG_DIR}/usr/local/bin/music-player"

# ─── DEBIAN/control ───────────────────────────────────
cat > "${PKG_DIR}/DEBIAN/control" << CONTROL
Package: ${APP_NAME}
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: ${ARCH}
Depends: python3 (>= 3.11), python3-venv, python3-pip, mpv, libmpv-dev
Maintainer: CLI Music Player <noreply@localhost>
Description: Terminal music player for Navidrome
 A beautiful TUI music player that connects to Navidrome instances
 via the Subsonic API and plays music through local speakers.
 Features include multi-server support, 18-band equalizer,
 shuffle, repeat, queue management, and live search.
Homepage: https://github.com/cli-music-player
CONTROL

# ─── DEBIAN/postinst (runs after install) ─────────────
cat > "${PKG_DIR}/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e

INSTALL_DIR="/opt/cli-music-player"
VENV_DIR="${INSTALL_DIR}/venv"

echo "Setting up CLI Music Player..."

# Create virtual environment
python3 -m venv "${VENV_DIR}"

# Install the package
source "${VENV_DIR}/bin/activate"
pip install --upgrade pip > /dev/null 2>&1
pip install --force-reinstall --no-cache-dir "${INSTALL_DIR}" > /dev/null 2>&1
deactivate

echo ""
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   🎵 CLI Music Player installed!         ║"
echo "  ║                                          ║"
echo "  ║   Run:  music-player                     ║"
echo "  ║   Help: music-player --help              ║"
echo "  ║                                          ║"
echo "  ║   First run: press S to add a server     ║"
echo "  ╚══════════════════════════════════════════╝"
echo ""
POSTINST
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# ─── DEBIAN/prerm (runs before removal) ───────────────
cat > "${PKG_DIR}/DEBIAN/prerm" << 'PRERM'
#!/bin/bash
set -e
# Clean up virtual environment
rm -rf /opt/cli-music-player/venv
echo "CLI Music Player removed."
PRERM
chmod 755 "${PKG_DIR}/DEBIAN/prerm"

# ─── Build the .deb ──────────────────────────────────
DEB_FILE="${SRC_DIR}/${APP_NAME}_${VERSION}_${ARCH}.deb"
dpkg-deb --root-owner-group --build "${PKG_DIR}" "${DEB_FILE}"

# ─── Cleanup ─────────────────────────────────────────
rm -rf "${PKG_DIR}"

echo ""
echo "✅ Package built: ${DEB_FILE}"
echo ""
echo "Install with:  sudo dpkg -i ${DEB_FILE}"
echo "Then run:       music-player"
