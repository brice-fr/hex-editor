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
if [[ $# -gt 0 ]]; then
  TAGS=("$@")
else
  TAGS=(v0.2.0 v0.2.1 v0.2.2)
fi

# Returns commits to revert (comma-separated) for a given tag, or empty string
tag_reverts() {
  case "$1" in
    v0.2.0) echo "b913542" ;;
    *)      echo "" ;;
  esac
}

# Source Rust + Node toolchains
source "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
[[ -s "$NVM_DIR/nvm.sh" ]] && source "$NVM_DIR/nvm.sh"

# ── Process each tag ──────────────────────────────────────────────────────────
for TAG in "${TAGS[@]}"; do
  info "══════════════════════════════════════════"
  info "Processing tag: $TAG"
  info "══════════════════════════════════════════"

  VERSION="${TAG#v}"
  WORK_DIR=$(mktemp -d)
  DMG_NAME="hex-editor_${VERSION}_aarch64.dmg"
  DMG_PATH="$WORK_DIR/src-tauri/target/release/bundle/dmg/$DMG_NAME"

  # Cleanup worktree on exit/error
  cleanup() {
    info "Cleaning up worktree …"
    git -C "$REPO_ROOT" worktree remove --force "$WORK_DIR" 2>/dev/null || true
    rm -rf "$WORK_DIR"
  }
  trap cleanup EXIT

  # Check the tag exists
  if ! git -C "$REPO_ROOT" rev-parse "$TAG" &>/dev/null; then
    warn "Tag $TAG not found locally — skipping (run: git fetch --tags)"
    trap - EXIT; cleanup
    continue
  fi

  # Create an isolated worktree for this tag (main working dir is untouched)
  info "Creating worktree for $TAG at $WORK_DIR …"
  git -C "$REPO_ROOT" worktree add --detach "$WORK_DIR" "$TAG"

  # Apply any required reverts on top of the tag (worktree only, not permanent)
  REVERTS_FOR_TAG="$(tag_reverts "$TAG")"
  if [[ -n "$REVERTS_FOR_TAG" ]]; then
    IFS=',' read -ra REVERTS <<< "$REVERTS_FOR_TAG"
    for COMMIT in "${REVERTS[@]}"; do
      info "Reverting $COMMIT on $TAG …"
      git -C "$WORK_DIR" revert --no-edit "$COMMIT"
    done
  fi

  # Build
  info "Installing npm dependencies …"
  (cd "$WORK_DIR" && npm install --silent)

  info "Building $TAG with signing + notarization (may take a few minutes) …"
  (cd "$WORK_DIR" && npm run tauri build)

  # Verify DMG was produced
  if [[ ! -f "$DMG_PATH" ]]; then
    error "DMG not found: $DMG_PATH"
    error "Check build output above. Skipping upload for $TAG."
    trap - EXIT; cleanup
    continue
  fi

  # Verify signature
  info "Verifying signature …"
  APP_PATH="$WORK_DIR/src-tauri/target/release/bundle/macos/hex-editor.app"
  if codesign -dv "$APP_PATH" 2>&1 | grep -q "Developer ID Application"; then
    info "Signature OK ✓"
  else
    warn "Signature check inconclusive — inspect manually: codesign -dv \"$APP_PATH\""
  fi

  # Upload to GitHub release
  if gh release view "$TAG" &>/dev/null; then
    info "Uploading $DMG_NAME to GitHub release $TAG …"
    gh release upload "$TAG" "$DMG_PATH" --clobber
    info "Uploaded ✓"
  else
    warn "No GitHub release found for $TAG — skipping upload"
    warn "DMG is at: $DMG_PATH (copy it before this script exits)"
    read -r -p "Press Enter to continue (and delete the worktree) …"
  fi

  trap - EXIT
  cleanup
done

# ── Done ──────────────────────────────────────────────────────────────────────
info "══════════════════════════════════════════"
info "All done."
