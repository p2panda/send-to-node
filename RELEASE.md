# Releasing send-to-node and send-blob-to-node

_This is an example for publising version `1.2.0`._

## Checks and preparations

1. Make sure you are on the `main` branch.

## Changelog time!

2. Check the git history for any commits on main that have not been mentioned
   in the _Unreleased_ section of `CHANGELOG.md` but should be.
3. Add an entry in `CHANGELOG.md` for this new release and move over all the
   _Unreleased_ stuff. Follow the formatting given by previous entries.

## Tagging and versioning

4. Bump the package version in `Cargo.toml` by hand.
5. Commit the version changes with a commit message `1.2.0`.
6. Run `git tag v1.2.0` and push including your tags using `git push origin
   main --tags`.

## Publishing releases

7. The GitHub Action will automatically create the release on GitHub, compile
   binary targets and upload them as assets. Check if the jobs succeeded.
