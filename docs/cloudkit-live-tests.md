# CloudKit live tests

The CloudKit sync live tests are opt-in because they write to iCloud.

Debug/non-production builds use the development container:

```text
iCloud.com.stboyden.mpalsts.dev
```

Release builds use the production container:

```text
iCloud.com.stboyden.mpalsts
```

## One-time Apple/Xcode setup

1. In Apple Developer, create or enable the iCloud container:

   ```text
   iCloud.com.stboyden.mpalsts.dev
   ```

2. Ensure the app/test bundle identifier used for live tests has the iCloud capability enabled:

   ```text
   com.stboyden.mpalsts.cloudkit-sync-tests
   ```

3. Enable CloudKit for the development container.

4. In Xcode, sign in to the Apple Developer account for the desired team:

   ```text
   Xcode > Settings > Accounts
   ```

5. Find the Team ID and run:

   ```sh
   DEVELOPMENT_TEAM=YOURTEAMID scripts/run-live-cloudkit-tests.sh
   ```

The script runs only the opt-in live Swift test and passes the dev CloudKit entitlements to `xcodebuild`.

## Rust live test

The Rust live test is gated behind the same environment variable:

```sh
MPALSTS_CLOUDKIT_LIVE_TESTS=1 cargo test -p mpalsts_sync live_cloudkit
```

For the Rust test binary to actually access CloudKit, it must also be signed with matching CloudKit entitlements. The Swift `xcodebuild` runner is currently the recommended way to validate the actual CloudKit write/read path.
