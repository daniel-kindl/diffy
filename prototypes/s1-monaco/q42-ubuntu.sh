#!/usr/bin/env bash
# Throwaway Q42 setup and run in an Ubuntu 24.04 distrobox container.
# On the host:  distrobox create -n diffy-ubuntu -i ubuntu:24.04 && distrobox enter diffy-ubuntu
# In the container, from this folder:  ./q42-ubuntu.sh
# Do not use the keyboard or the mouse during the run (about 8 minutes).
set -euo pipefail
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="$PWD/src-tauri/target-ubuntu"

sudo apt-get update -qq
sudo apt-get install -y -qq build-essential curl git pkg-config cmake libssl-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev \
  xdotool xclip wl-clipboard ibus ibus-gtk3 ibus-m17n ibus-libpinyin fonts-noto-cjk fontconfig

# ydotool 1.0.4, the same version as on Fedora. Ubuntu 24.04 has 0.1.x, which has a different command line.
if ! ydotool help 2>/dev/null | grep -q "mousemove"; then
  src="$(mktemp -d)"
  git clone --quiet --depth 1 --branch v1.0.4 https://github.com/ReimuNotMoe/ydotool "$src"
  cmake -S "$src" -B "$src/build" >/dev/null
  # Build only the binaries: the man pages need scdoc.
  make -C "$src/build" -j"$(nproc)" ydotool ydotoold >/dev/null
  sudo install "$src/build/ydotool" "$src/build/ydotoold" /usr/local/bin/
fi

# Node.js 24 and Rust stable for this container.
if ! node --version 2>/dev/null | grep -q '^v24'; then
  curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
  sudo apt-get install -y -qq nodejs
fi
if ! command -v cargo >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
. "$HOME/.cargo/env"

dpkg -s libwebkit2gtk-4.1-0 | grep '^Version'
[ -f fixtures/out/manifest.json ] || npm run fixtures
npx tauri build --no-bundle
./q42-run.sh ubuntu "$@"
