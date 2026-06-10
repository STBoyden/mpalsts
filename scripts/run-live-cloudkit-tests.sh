#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package_dir="$repo_root/swift/MPALSTSCloudKitSync"
entitlements="$package_dir/Entitlements/MPALSTSCloudKitSync.dev.entitlements"

if [[ -z "${DEVELOPMENT_TEAM:-}" ]]; then
  cat >&2 <<'EOF'
DEVELOPMENT_TEAM is required.

Find it in Xcode under Settings > Accounts, or from a signing identity such as:
  security find-identity -v -p codesigning

Then run, for example:
  DEVELOPMENT_TEAM=YOURTEAMID scripts/run-live-cloudkit-tests.sh
EOF
  exit 2
fi

if ! /usr/libexec/PlistBuddy -c 'Print :com.apple.developer.icloud-container-identifiers:0' "$entitlements" >/dev/null; then
  echo "Could not read CloudKit entitlements at $entitlements" >&2
  exit 1
fi

cd "$package_dir"

MPALSTS_CLOUDKIT_LIVE_TESTS=1 xcodebuild \
  -scheme MPALSTSCloudKitSync \
  -destination 'platform=macOS' \
  -configuration Debug \
  -allowProvisioningUpdates \
  DEVELOPMENT_TEAM="$DEVELOPMENT_TEAM" \
  CODE_SIGN_STYLE=Automatic \
  CODE_SIGN_IDENTITY='Apple Development' \
  PRODUCT_BUNDLE_IDENTIFIER='com.stboyden.mpalsts.cloudkit-sync-tests' \
  CODE_SIGN_ENTITLEMENTS="$entitlements" \
  OTHER_CODE_SIGN_FLAGS="--entitlements $entitlements" \
  test \
  -only-testing:MPALSTSCloudKitSyncTests/liveCloudKitPushesDarkAppearanceAndReadsItBack
