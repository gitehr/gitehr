# dist - packaging artefacts

Supplementary packaging for channels cargo-dist does not produce.

- `PKGBUILD` - Arch Linux (AUR) package build recipe. Version and checksums are
  refreshed from each release's `sha256.sum` (the single source of truth) at
  release time; publishing requires an AUR account (R55 blocked on that).
- Scoop manifest lives in [pacharanero/scoop](https://github.com/pacharanero/scoop)
  (`bucket/gitehr.json`, with `checkver`/`autoupdate` driven by release
  checksums, same pattern as `sct.json`).
- Homebrew formula lives in [pacharanero/homebrew-tap](https://github.com/pacharanero/homebrew-tap)
  (`Formula/gitehr.rb`), updated by cargo-dist on each release.
- APT/RPM (R54) need a hosted repository with GPG signing infrastructure;
  not yet actionable until signing keys and repo hosting are decided.
