# itr06 plan H: retake the screenshots in CI (I18)

Goal: nobody retakes `docs/images` by hand. A workflow takes the screenshots
when the UI changes and opens one PR with the images that really changed.

Tasks:
1. `scripts/site/compare-screenshots.sh <new> [<ref>]`: ImageMagick
   `compare -metric AE -fuzz <n>%` per PNG; options `--fuzz` (default 5%),
   `--threshold` (default 50 pixels), `--apply`, `--prune`; lists changed,
   added and removed images. Verified on identical and modified copies.
2. `.github/workflows/screenshots.yml`: workflow_dispatch and push to master
   on UI paths; runs `screenshots.sh --out`, compares, and opens or updates
   the PR from `docs/retake-screenshots` with `gh`. Optional
   `SCREENSHOTS_TOKEN` secret (so CI runs on the PR), else `GITHUB_TOKEN`.
3. Docs: CLAUDE.md (Commands, Testing notes), `docs/technical/release-process.md`,
   PR template checklist.
