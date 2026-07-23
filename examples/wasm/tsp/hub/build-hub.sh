#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HUB_DIR="$ROOT_DIR/hub"
OUT_DIR="$HUB_DIR/site"
LOCAL_APPS_DIR="$HUB_DIR/apps"

apps=(vanilla react yew leptos)

echo "Building framework apps..."
for app in "${apps[@]}"; do
  APP_DIR="$ROOT_DIR/$app/app"
  echo "- building $app"
  (
    cd "$APP_DIR"
    npm run build
  )
done

echo "Preparing hub site output..."
rm -rf "$OUT_DIR"
rm -rf "$LOCAL_APPS_DIR"
mkdir -p "$OUT_DIR/apps"
mkdir -p "$LOCAL_APPS_DIR"
cp "$HUB_DIR/index.html" "$OUT_DIR/index.html"
cp "$HUB_DIR/style.css" "$OUT_DIR/style.css"
cp "$HUB_DIR/favicon.ico" "$OUT_DIR/favicon.ico"
cp -r "$HUB_DIR/assets" "$OUT_DIR/assets"

for app in "${apps[@]}"; do
  SRC_DIR="$ROOT_DIR/$app/app/dist"
  DST_DIR="$OUT_DIR/apps/$app"
  LOCAL_DST_DIR="$LOCAL_APPS_DIR/$app"
  mkdir -p "$DST_DIR"
  mkdir -p "$LOCAL_DST_DIR"
  cp -r "$SRC_DIR"/. "$DST_DIR"/
  cp -r "$SRC_DIR"/. "$LOCAL_DST_DIR"/
done

echo "Hub site generated at: $OUT_DIR"
echo "Direct-launch apps generated at: $LOCAL_APPS_DIR"
echo "Use an http/https server to run the hub (do not open with file://)."
