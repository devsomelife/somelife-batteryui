#!/usr/bin/env bash
#
# Cut a release: bump the version, build, commit, push, tag, and push the tag.
# Pushing the v* tag triggers .github/workflows/release.yml, which builds the
# musl binary + .deb and publishes the GitHub Release.
#
# Usage: scripts/release.sh <version>   (e.g. scripts/release.sh 1.2.0)
# The version must be bare MAJOR.MINOR.PATCH, without a leading "v".

set -euo pipefail

step() { printf '\n==> %s\n' "$1"; }
die() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

# --- arg + format check ----------------------------------------------------
[ "$#" -eq 1 ] || die "usage: scripts/release.sh <version>  (e.g. 1.2.0)"
VERSION="$1"

case "$VERSION" in
v*) die "drop the leading 'v' — pass ${VERSION#v}, the tag is added automatically" ;;
esac
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
  die "version must be MAJOR.MINOR.PATCH (digits only), got '$VERSION'"

TAG="v$VERSION"

# --- toolchain + repo root -------------------------------------------------
# Use the rustup cargo (the committed Cargo.lock is v4; system cargo 1.75 fails).
# shellcheck disable=SC1090
. "$HOME/.cargo/env"
cd "$(dirname "$0")/.."

# --- preconditions ---------------------------------------------------------
step "Checking working tree and branch"
git diff --quiet && git diff --cached --quiet ||
  die "working tree is dirty — commit or stash first"

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" = "main" ] || die "must release from 'main', currently on '$BRANCH'"

git fetch origin
[ "$(git rev-parse @)" = "$(git rev-parse '@{u}')" ] ||
  die "local main is not in sync with origin/main — pull/push first"

step "Checking that tag $TAG does not already exist"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  die "tag $TAG already exists locally"
fi
if git ls-remote --exit-code --tags origin "$TAG" >/dev/null 2>&1; then
  die "tag $TAG already exists on origin"
fi

CURRENT="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
[ "$CURRENT" != "$VERSION" ] ||
  die "Cargo.toml is already at version $VERSION"

# --- bump version ----------------------------------------------------------
step "Bumping version $CURRENT -> $VERSION in Cargo.toml"
sed -i "0,/^version = \".*\"/s//version = \"$VERSION\"/" Cargo.toml

# --- build (refreshes Cargo.lock) + quality gates --------------------------
step "Building release (refreshes Cargo.lock)"
cargo build --release

step "Checking formatting and lints"
cargo fmt --check
cargo clippy --all-targets -- -D warnings

# --- commit, push, tag, push tag -------------------------------------------
step "Committing"
git add Cargo.toml Cargo.lock
git commit -m "chore: release $VERSION"

step "Pushing main"
git push origin main

step "Tagging $TAG and pushing it"
git tag "$TAG"
git push origin "$TAG"

# --- done ------------------------------------------------------------------
REPO_URL="https://github.com/devsomelife/somelife-batteryui"
cat <<EOF

Released $TAG. The release workflow is now building the musl binary and .deb.

Check progress:
  gh run list --workflow=release.yml
  gh run watch
View the release once published:
  gh release view $TAG
  $REPO_URL/releases/tag/$TAG
EOF
