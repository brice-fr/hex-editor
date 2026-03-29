#!/usr/bin/env bash
# resign-releases.sh — rebuild old release tags with macOS signing and upload
# to the existing GitHub releases.
#
# Prerequisites:
#   export APPLE_SIGNING_IDENTITY="Developer ID Application: ..."
#   export APPLE_ID="your@appleid.com"
#   export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"
#   export APPLE_TEAM_ID="XXXXXXXXXX"
#
# Usage: bash scripts/resign-releases.sh [v0.2.0 v0.2.1 v0.2.2]
#        (defaults to all three if no args given)

set -euo pipefail

# ── Colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()    { echo -e "${GREEN}[INFO]${NC}  $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# ── Guard: required env vars ──────────────────────────────────────────────────
required_vars=(APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID)
missing=()
for v in "${required_vars[@]}"; do
  [[ -z "${!v:-}" ]] && missing+=("$v")
done
if [[ ${#missing[@]} -gt 0 ]]; then
  error "Missing required environment variables:"
  for v in "${missing[@]}"; do error "  $v"; done
  echo ""
  echo "Set them first, e.g.:"
  echo '  export APPLE_SIGNING_IDENTITY="Developer ID Application: John Doe (ABCD123456)"'
  echo '  export APPLE_ID="your@appleid.com"'
  echo '  export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"'
  echo '  export APPLE_TEAM_ID="ABCD123456"'
  exit 1
fi

# ── Setup ─────────────────────────────────────────────────────────────────────
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAGS=("${@:-v0.2.0 v0.2.1 v0.2.2}")
ORIGINAL_BRANCH=$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD)

# Source Rust + Node toolchains
source "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
[[ -s "$NVM_DIR/nvm.sh" ]] && source "$NVM_DIR/nvm.sh"

cd "$REPO_ROOT"

# ── Process each tag ──────────────────────────────────────────────────────────
for TAG in "${TAGS[@]}"; do
  info "══════════════════════════════════════════"
  info "Processing tag: $TAG"
  info "══════════════════════════════════════════"

  # Extract semver from tag (strip leading 'v')
  VERSION="${TAG#v}"
  DMG_NAME="hex-editor_${VERSION}_aarch64.dmg"
  DMG_PATH="$REPO_ROOT/src-tauri/target/release/bundle/dmg/$DMG_NAME"

  # Check the tag exists
  if ! git rev-parse "$TAG" &>/dev/null; then
    warn "Tag $TAG not found locally — skipping"
    continue
  fi

  # Check the GitHub release exists
  if ! gh release view "$TAG" &>/dev/null; then
    warn "No GitHub release found for $TAG — skipping upload (build will still run)"
  fi

  # Stash any uncommitted changes, check out tag
  git stash --quiet 2>/dev/null || true
  info "Checking out $TAG …"
  git checkout --quiet "$TAG"

  # Clean previous bundle artifacts to avoid stale DMG
  rm -f "$DMG_PATH"

  # Build with signing
  info "Building $TAG (this includes notarization — may take a few minutes) …"
  npm install --silent
  npm run tauri build

  # Verify DMG was produced
  if [[ ! -f "$DMG_PATH" ]]; then
    error "DMG not found at expected path: $DMG_PATH"
    error "Check build output above. Skipping upload for $TAG."
    git checkout --quiet "$ORIGINAL_BRANCH"
    continue
  fi

  # Verify the app is signed and notarized
  info "Verifying signature …"
  APP_PATH="$REPO_ROOT/src-tauri/target/release/bundle/macos/hex-editor.app"
  if codesign -dv --verbose=2 "$APP_PATH" 2>&1 | grep -q "Developer ID Application"; then
    info "Signature OK"
  else
    warn "Signature check inconclusive — inspect manually with: codesign -dv $APP_PATH"
  fi

  # Upload to GitHub release
  if gh release view "$TAG" &>/dev/null; then
    info "Uploading $DMG_NAME to GitHub release $TAG …"
    gh release upload "$TAG" "$DMG_PATH" --clobber
    info "Uploaded successfully."
  else
    warn "Skipping upload — no GitHub release for $TAG"
  fi

  # Return to original branch before next iteration
  git checkout --quiet "$ORIGINAL_BRANCH"
done

# ── Done ──────────────────────────────────────────────────────────────────────
info "══════════════════════════════════════════"
info "All done. Returning to branch: $ORIGINAL_BRANCH"
git checkout --quiet "$ORIGINAL_BRANCH"
