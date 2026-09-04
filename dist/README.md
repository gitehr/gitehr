# dist - packaging artefacts

Supplementary packaging for channels cargo-dist does not produce.

- `PKGBUILD` - Arch Linux (AUR) package build recipe. The `publish-aur-package`
  job in `.github/workflows/release.yml` clones `aur@aur.archlinux.org:gitehr.git`
  on each release using the `AUR_SSH_PRIVATE_KEY` secret, sets `pkgver` and the
  per-arch `sha256sums` from the release `sha256.sum`, regenerates `.SRCINFO`,
  and pushes - the same release-sha256-as-source-of-truth convention as the
  Scoop manifest.
- Scoop manifest lives in [pacharanero/scoop](https://github.com/pacharanero/scoop)
  (`bucket/gitehr.json`, with `checkver`/`autoupdate` driven by release
  checksums, same pattern as `sct.json`).
- Homebrew formula lives in [pacharanero/homebrew-tap](https://github.com/pacharanero/homebrew-tap)
  (`Formula/gitehr.rb`), updated by cargo-dist on each release.
- APT/RPM (R54) need a hosted repository with GPG signing infrastructure;
  not yet actionable until signing keys and repo hosting are decided.
