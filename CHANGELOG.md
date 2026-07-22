# Changelog

All notable changes to GitEHR are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Releases are grouped from conventional-commit messages by [git-cliff](https://git-cliff.org).

## [0.3.6] - 2026-07-22

### Bug fixes

- **docs**: Repair root-owned generated site and cache before preview ([abc196d](https://github.com/gitehr/gitehr/commit/abc196da1138a669d2f8083ffe7f4057699f04f5))

### CI

- Align workflows with house-style security and crates.io Trusted Publishing ([3eb9dab](https://github.com/gitehr/gitehr/commit/3eb9dabc44d5c5d9946b63564e7d6e2a1760cc83))

- **release**: Publish and test Homebrew formulae ([8cc6604](https://github.com/gitehr/gitehr/commit/8cc66042528ba842c46c9ffa88d101958a8ebde8))

### CI / dependencies

- **deps**: Bump clap from 4.6.2 to 4.6.3 ([#107](https://github.com/gitehr/gitehr/issues/107)) ([802c999](https://github.com/gitehr/gitehr/commit/802c999cbea17527cb75bd3ca7b0ad292c74e971))

- **deps**: Bump tokio from 1.53.0 to 1.53.1 ([#108](https://github.com/gitehr/gitehr/issues/108)) ([a04bee5](https://github.com/gitehr/gitehr/commit/a04bee548381ccd3171acf909837fccfb5b1db78))

### Documentation

- **roadmap**: Remove completed items, reorder, and add spec gaps ([e869bfb](https://github.com/gitehr/gitehr/commit/e869bfbf50117ca95daaf672b0076de5aaafda4a))

- **adr**: Multiple Stores are a GUI concern (ADR-0006) ([914a9f0](https://github.com/gitehr/gitehr/commit/914a9f01aed08556cb52d055c6e4f89f39e41582))

- **install**: Lead with tabbed prebuilt binary options and OS icons ([7da1d59](https://github.com/gitehr/gitehr/commit/7da1d59b140188e5461552fc805624f95c8a866d))

## [0.3.5] - 2026-07-20

### Bug fixes

- **release**: Skip unavailable Homebrew publisher ([4d6f787](https://github.com/gitehr/gitehr/commit/4d6f7870c062d4333b84d3c195dd43c85ce228ab))

## [0.3.4] - 2026-07-20

### Bug fixes

- **release**: Defer Homebrew publishing ([0e0dca8](https://github.com/gitehr/gitehr/commit/0e0dca84b50498744098e499a9e38170e368a1da))

## [0.3.3] - 2026-07-20

### Bug fixes

- **release**: Generate WiX installer metadata ([bd1dd3c](https://github.com/gitehr/gitehr/commit/bd1dd3c11f4481954a6e2176e83d70d634bc2448))

## [0.3.2] - 2026-07-20

### Bug fixes

- **docs**: Run Zensical as the caller ([4a1e810](https://github.com/gitehr/gitehr/commit/4a1e810c9633188b3cc0531c0ae5020502001147))

- **mcp**: Remove unavailable journal verifier ([286f524](https://github.com/gitehr/gitehr/commit/286f52477e8d59ac2d67b867953f5a144abc8d22))

- **store**: Require an empty store root ([d2e1866](https://github.com/gitehr/gitehr/commit/d2e1866b86ef30a8defec19e10ebf19bea450949))

- **clincalc**: Rename calculator plugin ([1136fd4](https://github.com/gitehr/gitehr/commit/1136fd4157fef99e160bf949ae9cc3fae88416dc))

- **cli**: Make command help discoverable ([afe8036](https://github.com/gitehr/gitehr/commit/afe80363881d32a2292e5d14e9ca875b4863b78a))

- **docs**: Clean site before zensical builds ([07732c6](https://github.com/gitehr/gitehr/commit/07732c606718706aa24f4fb3f71c354ba301d582))

### CI

- **release**: Publish gitehr to crates.io ([98136f9](https://github.com/gitehr/gitehr/commit/98136f9a15c581fab93af0e4ea6e43059ad90385))

- **release**: Restore house-style release flow ([2c54171](https://github.com/gitehr/gitehr/commit/2c54171b77a9138280f9c20cb778fb476b1498e7))

### CI / dependencies

- **deps**: Bump dtolnay/rust-toolchain ([#104](https://github.com/gitehr/gitehr/issues/104)) ([8fb2ddc](https://github.com/gitehr/gitehr/commit/8fb2ddcffbf59aaff3ab49c81ae7837f809a1f8a))

- **deps**: Bump toml from 1.1.2+spec-1.1.0 to 1.1.3+spec-1.1.0 ([#103](https://github.com/gitehr/gitehr/issues/103)) ([f3d19de](https://github.com/gitehr/gitehr/commit/f3d19de2e4bcfc64a703eac604d50b6d31bbbdce))

- **deps**: Bump uuid from 1.23.4 to 1.23.5 ([#101](https://github.com/gitehr/gitehr/issues/101)) ([4602a51](https://github.com/gitehr/gitehr/commit/4602a51f139bf6721cf00dc5b08d3826024548c3))

- **deps**: Bump regex from 1.12.4 to 1.13.0 ([#99](https://github.com/gitehr/gitehr/issues/99)) ([1116f47](https://github.com/gitehr/gitehr/commit/1116f4762e3f927b202fae29f9211148cae9f9b4))

- **deps**: Bump clap_complete from 4.6.5 to 4.6.7 ([#97](https://github.com/gitehr/gitehr/issues/97)) ([38a629e](https://github.com/gitehr/gitehr/commit/38a629e9bc80ccfa9a7f611738e5fc29c5e65f3e))

- **deps**: Bump rand from 0.10.1 to 0.10.2 ([#96](https://github.com/gitehr/gitehr/issues/96)) ([e8648b3](https://github.com/gitehr/gitehr/commit/e8648b3d2bfc33b85e6ce68388af5eca24bdf54e))

### Chores

- **gui**: Sync version metadata ([3053ed7](https://github.com/gitehr/gitehr/commit/3053ed78405fcf08b873f78b6917d2ee154605d0))

### Documentation

- Add Common objections page covering concurrent edits, ACID, and GDPR erasure ([7d44339](https://github.com/gitehr/gitehr/commit/7d44339ee09412ee8b5f75b01676c9716425eecd))

- **roadmap**: Focus outstanding distribution work ([30f13a9](https://github.com/gitehr/gitehr/commit/30f13a94a93be2ffe946475eae435e8bfb5e62a2))

- **import**: Propose NHS App extraction ([fdf932f](https://github.com/gitehr/gitehr/commit/fdf932f7e13af336ffdd6bd783a8adbe36fb4391))

- **design**: Add archival and query boundaries ([988d621](https://github.com/gitehr/gitehr/commit/988d62196bc513f6011cec7d6d17d3221327a309))

- **journal**: Correct Git integrity guarantees ([d08677c](https://github.com/gitehr/gitehr/commit/d08677ccc38d924d13b68701ad0773372a7dc698))

- **design**: Illustrate interoperability scaling ([e0159e9](https://github.com/gitehr/gitehr/commit/e0159e95df56010c052c18131e704d13ec364094))

- **design**: Explain agent-readable records ([35cb036](https://github.com/gitehr/gitehr/commit/35cb03652780fd54d5eb305422469e24ee6cafad))

- **spec**: Draft condition and provenance models ([7a37a4c](https://github.com/gitehr/gitehr/commit/7a37a4ca13e597381f6d79da048ded7f9c8d57ae))

### Features

- **state**: Add typed vaccinations ([f4a2b85](https://github.com/gitehr/gitehr/commit/f4a2b8520bc6cc4a2add0ba0df3ee736565225ed))

- **gui**: Attach documents and show patient state ([86b3995](https://github.com/gitehr/gitehr/commit/86b399542886618e5429ce4a8d53cce40c803a81))

- **state**: Add typed demographics and allergies ([701b2cc](https://github.com/gitehr/gitehr/commit/701b2cc5a094838408280069900cbd4cd9a73c80))

- **document**: Attach multiple documents per entry ([29bdab4](https://github.com/gitehr/gitehr/commit/29bdab48fbd066e16dd66d25acc3c8570e656b71))

### Merge

- Integrate origin main ([36cfbf6](https://github.com/gitehr/gitehr/commit/36cfbf600b7dc2ec6c6260b59096fb2475da2b14))

## [0.3.1] - 2026-06-29

### Chores

- Release v0.3.0 ([57087be](https://github.com/gitehr/gitehr/commit/57087bea8df2c51347150d17e5368068955c194b))

## [0.3.0] - 2026-06-29

### CI / dependencies

- **deps**: Bump anyhow from 1.0.102 to 1.0.103 ([4f14bc8](https://github.com/gitehr/gitehr/commit/4f14bc87e29b62e84713768a4866866c8e87bdda))

- **deps**: Bump Swatinem/rust-cache ([9a91b1e](https://github.com/gitehr/gitehr/commit/9a91b1e1881ea7cf555a5a5bf479cc3b8dc4c4b5))

- **deps**: Bump toml from 0.9.12+spec-1.1.0 to 1.1.2+spec-1.1.0 ([7235514](https://github.com/gitehr/gitehr/commit/7235514cda5eb5fb432e1bce0617bd62ed2956c0))

### Chores

- Release v0.3.0 ([1dc076e](https://github.com/gitehr/gitehr/commit/1dc076ea59deea1480465b86d322adc48c292c5b))

### Features

- **cli**: Add completions installer ([c28f282](https://github.com/gitehr/gitehr/commit/c28f28233fd084349a94ce31855e1628024dd4b7))

## [0.2.2] - 2026-06-28

### Bug fixes

- **ci**: Satisfy clippy for config ([f58eada](https://github.com/gitehr/gitehr/commit/f58eada0cba2f7ab29d71ac5f05ccf6bd1b7ccd8))

### Chores

- Release v0.2.2 ([caf75b7](https://github.com/gitehr/gitehr/commit/caf75b7fd0a331596b49e13c9f59ea74cb129292))

### Documentation

- **mcp**: Describe mcp as gitehr server mode ([d54a25f](https://github.com/gitehr/gitehr/commit/d54a25f82f2416f410639f8cdc0abe85b685aef8))

### Features

- **config**: Add default store config ([219590c](https://github.com/gitehr/gitehr/commit/219590c3f3b7b7513694ffa25057e47809b48b57))

### Refactor

- **mcp**: Fold server into gitehr binary ([8849620](https://github.com/gitehr/gitehr/commit/884962082b7a98dd0dea61c4356c6ce3849178fd))

## [0.2.1] - 2026-06-27

### Bug fixes

- **release**: Detach unpublished package deps temporarily ([6c2ee90](https://github.com/gitehr/gitehr/commit/6c2ee90e9a680403ad8fef8f88cb57f7830632bb))

- **gui**: Align desktop app with store-first CLI ([eec9068](https://github.com/gitehr/gitehr/commit/eec9068d183e82e1dc0e3ef4967e9d3a30894fdc))

- **gui**: Resolve committed conflict markers in package-lock.json ([08a0277](https://github.com/gitehr/gitehr/commit/08a0277b09c9dfdba7fdeb14b5104aaaa89b1df8))

### Build

- Track gui/src-tauri/Cargo.lock too ([c817a2b](https://github.com/gitehr/gitehr/commit/c817a2bd19e656de5003af260606f6fc547f6751))

- Track Cargo.lock for reproducible builds ([3eb84f9](https://github.com/gitehr/gitehr/commit/3eb84f9bd9f80d83327cbaaa6b2f2202c5795666))

### CI

- **release**: Fix release-plz repo activation ([44707c2](https://github.com/gitehr/gitehr/commit/44707c2d7de5956d4e3a56d6ff6a41c3dd696931))

- **release**: Make release-plz canonical ([d13dc65](https://github.com/gitehr/gitehr/commit/d13dc6579ad79480abeacc809c7d1622c2f8e118))

- Trial release-plz for automated version, changelog, and releases ([47e8345](https://github.com/gitehr/gitehr/commit/47e8345a4c83ad3fa14f7ca3e04368bb2549721c))

### CI / dependencies

- **deps**: Bump actions/setup-python from 6.2.0 to 6.3.0 ([#79](https://github.com/gitehr/gitehr/issues/79)) ([c081e3c](https://github.com/gitehr/gitehr/commit/c081e3cb4001df2f8340d8e7d2c6ce179abaaa7e))

### Documentation

- Fix the Zensical build (move PDF extracts out of docs/, fix dead anchors) ([591ce4c](https://github.com/gitehr/gitehr/commit/591ce4cef5a3b21359a12ddf2b8f98d2c9fd1a6d))

- Retarget everything at `gitehr store` after init removal (ADR-0005) ([42808f6](https://github.com/gitehr/gitehr/commit/42808f6e26b53348d7b37d46e8fac60e5566509e))

- **spec**: ADR-0005 - GitEHR is Store-first ([ab7ada4](https://github.com/gitehr/gitehr/commit/ab7ada4e83b86a67c49c5fe92683e5a788420337))

- **cli**: Document the import command; roadmap a built-in OCR import feature ([3a34d93](https://github.com/gitehr/gitehr/commit/3a34d93446f9a5c07c5fbc36d88b690c98da0636))

### Features

- **cli**: Collapse journal entry creation into one `journal add` **[breaking]** ([2c395a3](https://github.com/gitehr/gitehr/commit/2c395a309695a7b67f27e2f1ebd28acb661675a8))

- **cli**: Remove top-level `gitehr init` (ADR-0005, part 3) **[breaking]** ([4b86c82](https://github.com/gitehr/gitehr/commit/4b86c829aa59b4ab404a7632da0e523d8330621e))

- **cli**: Store/repo context detection + auto-target (ADR-0005, part 2) ([3f284fc](https://github.com/gitehr/gitehr/commit/3f284fcd7e482e90a4d431094d25c3df6bc38adc))

- **cli**: Store-first store commands (ADR-0005, part 1) ([98cfedf](https://github.com/gitehr/gitehr/commit/98cfedf7bc55b7b9ced1f9bb249f3c6cb3a99797))

- **cli**: Add import command for journal entries and documents ([#76](https://github.com/gitehr/gitehr/issues/76)) ([fab0271](https://github.com/gitehr/gitehr/commit/fab0271238b442c52e3ad3b1022cb0a40feaa8a8))

### Refactor

- **cli**: Alias `contributor` to `user`; sync docs + roadmap ([a84b999](https://github.com/gitehr/gitehr/commit/a84b99945685b4661035bf6618a11df69293797a))

- **cli**: Flatten `gitehr server` into top-level `user` and `store` ([f0c32ab](https://github.com/gitehr/gitehr/commit/f0c32ab600d7b37c2a6557b856ebfab79f6fd297))

## [0.2.0] - 2026-06-25

### Bug fixes

- **docs**: Actually hide the homepage nav under the modern theme variant ([f674335](https://github.com/gitehr/gitehr/commit/f67433560b7c3b6b12193c13a58884ddb85f5c14))

- **docs**: Stop the nav sidebar overlapping the homepage hero ([f0078b7](https://github.com/gitehr/gitehr/commit/f0078b7fa1062955560040eb216da5f87870af55))

### Build

- **deps**: Complete the calc dep repoint to public pacharanero/calc ([ce7caef](https://github.com/gitehr/gitehr/commit/ce7caeffd5c6e5b0b55b622b30a2560629d3f2cb))

- **deps**: Point calc dependency at the public pacharanero/calc repo ([8ef89b4](https://github.com/gitehr/gitehr/commit/8ef89b4456ee3976e8ec2265441a7b62ce8fb45a))

- **calc**: Consume calc-cli/calc-core as a git dep on gitehr/tools ([8dcc56f](https://github.com/gitehr/gitehr/commit/8dcc56fbc689a698d10783715eae5178a21edd00))

### CI

- Add fmt/clippy/test workflow ([34bee0d](https://github.com/gitehr/gitehr/commit/34bee0dd9f46505a817fa68f6be8647eedebc4a4))

### CI / dependencies

- **deps**: Bump which from 8.0.3 to 8.0.4 ([#68](https://github.com/gitehr/gitehr/issues/68)) ([7b0ba25](https://github.com/gitehr/gitehr/commit/7b0ba258966260badd9cce4ce795f03a13817e9f))

- **deps**: Bump tar from 0.4.44 to 0.4.46 ([#67](https://github.com/gitehr/gitehr/issues/67)) ([8136901](https://github.com/gitehr/gitehr/commit/813690108067ad0377c41760c6633b6fd6313d9f))

- **deps**: Bump regex from 1.12.3 to 1.12.4 ([#66](https://github.com/gitehr/gitehr/issues/66)) ([1969460](https://github.com/gitehr/gitehr/commit/1969460669eb8e4ecae7812ed13dfaadc3ef20ad))

- **deps**: Bump actions/checkout from 6.0.3 to 7.0.0 ([afc5042](https://github.com/gitehr/gitehr/commit/afc504228a27083a0828eb4b80450c6ff158d583))

- **deps**: Bump sha2 0.10 -> 0.11 ([a207729](https://github.com/gitehr/gitehr/commit/a207729e38d15efce1bb1fd3ef34c6c44811406c))

- **deps**: Bump chrono, uuid, which, serial_test (patch) ([f116f95](https://github.com/gitehr/gitehr/commit/f116f959749100873fc5c1db81b56d1c61dda437))

- **deps**: Apply pending Dependabot updates ([3ca345e](https://github.com/gitehr/gitehr/commit/3ca345e56b2f3b8e484a51bfdde1e3881422b869))

### Chores

- Untrack Cargo.lock ([c89e4ac](https://github.com/gitehr/gitehr/commit/c89e4ac818170cef4f9afc4a82aca8ba417ec0a1))

- **license**: Add SPDX headers to cli/mcp sources and s/ scripts ([95a355f](https://github.com/gitehr/gitehr/commit/95a355fc046b2f0ec9bc9265b0807a53ff13cea2))

- Bump version to 0.2.0 ([43725ec](https://github.com/gitehr/gitehr/commit/43725ec503f8971caf74215a43638dece5f220c1))

- Clear restructure warnings and apply rustfmt ([e5beb10](https://github.com/gitehr/gitehr/commit/e5beb1061fa5fad85a1b7d86d4b66dcbe1ba7160))

### Documentation

- **journal**: Rewrite spec to match the current command surface ([cce2b9c](https://github.com/gitehr/gitehr/commit/cce2b9c7ace39a8a2446329722284a050551b752))

- **journal**: Document LATEST relative entry reference syntax ([#75](https://github.com/gitehr/gitehr/issues/75)) ([217e5e4](https://github.com/gitehr/gitehr/commit/217e5e4fdf2d87c617abac71f0ffe5543de35a94))

- Add Stonebraker rebuttal + DB-vs-files evidence; roadmap gittuf review ([9a905bb](https://github.com/gitehr/gitehr/commit/9a905bb69bd4c9904d70cb31e27228dd79609b81))

- Add database-vs-files reference PDFs + extracted markdown + analysis note ([6d722c3](https://github.com/gitehr/gitehr/commit/6d722c39efa70cfac6d25cc3f3db1907e0431ed7))

- **spec**: Fold docs/.todo.md points into the spec/ files; remove the scratch list ([02cd585](https://github.com/gitehr/gitehr/commit/02cd585cb38c50563ac1fa4dca508cd0d6bed385))

- **calc**: Update calculator list for the complete library + proprietary tools ([9b96622](https://github.com/gitehr/gitehr/commit/9b9662261d854d063eda2b9d15318737a8bc58f6))

- **roadmap**: Mark the 50-tool calculator library complete ([0309262](https://github.com/gitehr/gitehr/commit/0309262d9e7427b693481113577858b688fe51d8))

- **spec**: Design Medical Markdown integration; ADR-0004 derived structured data ([f520ac5](https://github.com/gitehr/gitehr/commit/f520ac58621be2b6e7e254215ea6d68e148a2cd4))

- **spec**: Add encryption-at-rest research doc; move ADRs to spec/ ([8f0fd39](https://github.com/gitehr/gitehr/commit/8f0fd39be59dae6b3a7d2ff12be17690a7cc5e6f))

### Features

- **cli**: Add Git-style $PATH plugin extensibility ([c393ee3](https://github.com/gitehr/gitehr/commit/c393ee3d9fcfb2680dab7f6ddd48dbdb604aea02))

- **s**: Add s/lint ([6f92f22](https://github.com/gitehr/gitehr/commit/6f92f222488be0d719086c005037aaf10e058afb))

- **s**: Add s/test ([47f8b3e](https://github.com/gitehr/gitehr/commit/47f8b3e135ade84c23bfb596151cd6fd86ddcecb))

- **s**: Add s/size disk-footprint report ([65c3673](https://github.com/gitehr/gitehr/commit/65c36739270aee6bda4b75f8ffcea7cd9a5c1323))

- **calc**: Add QRISK3 and QFracture (LGPL, validated against ClinRisk) ([65d3d51](https://github.com/gitehr/gitehr/commit/65d3d517627345d5d43e617f621c1c487ac3535c))

- **calc**: Add Gleason, NPI, CHALICE, KDIGO CKD-risk, GRACE, EuroSCORE II ([4103da3](https://github.com/gitehr/gitehr/commit/4103da331fcadedde0b51372a41c16584d97dec8))

- **calc**: Add Padua, UKELD, NHFS, BODE, ABPI, Waterlow; CFS + LANSS stubs ([51a29d0](https://github.com/gitehr/gitehr/commit/51a29d0e537a284b7b8678be936bcd7d93ab0744))

- **calc**: Add DAS28, uACR, SOFA, HEART, TIMI, Child-Pugh, MELD; MUST stub ([518d2d0](https://github.com/gitehr/gitehr/commit/518d2d0b3ccfc9b09ca0341b8dcca5d244dc7eb9))

- **calc**: Add NEWS2, CURB-65, Wells DVT/PE, HAS-BLED, ABCD2, qSOFA, 4AT ([c03a94f](https://github.com/gitehr/gitehr/commit/c03a94f662eed4dcfc66483dd9c2b080c15f0fde))

- **calc**: Add AUDIT-C, AUDIT, EPDS, IPSS, AMTS, MRC Dyspnoea; CAT stub ([33fde7c](https://github.com/gitehr/gitehr/commit/33fde7c0693de4f3f1390b2a53f6749dc894b840))

- **calc**: Protest stubs for proprietary / licence-locked calculators ([2d3ab05](https://github.com/gitehr/gitehr/commit/2d3ab05504d1478321d1c73ddc1a214dd765d5c7))

- **calc**: Add CHA2DS2-VASc with the full input-definition treatment ([d786ff3](https://github.com/gitehr/gitehr/commit/d786ff3d89b9c4c2ff9111555017de0984f6dcab))

- **calc**: Require a distribution licence + evidence URL per calculator ([f6ed566](https://github.com/gitehr/gitehr/commit/f6ed566ef768dc958bbc084e9ad5d956f4d89cd3))

- **calc**: Add eGFR (CKD-EPI 2021) and FIB-4 calculators ([5f044eb](https://github.com/gitehr/gitehr/commit/5f044eb1e54cf43f170d6d264b08f539a3c19cd9))

- **calc**: Add PHQ-9 and GAD-7; design input-definition system; calc docs ([15e3187](https://github.com/gitehr/gitehr/commit/15e318792abdda42660fc87f367994a0d8d3b029))

- **calc**: Replace per-calculator flags with one JSON template surface ([4152d24](https://github.com/gitehr/gitehr/commit/4152d243bbd399eb5f762ba4f93332813123be27))

- **calc**: Wire clinical calculators into gitehr CLI and MCP server ([caffc75](https://github.com/gitehr/gitehr/commit/caffc75692774e54d850a7d4d313735647692fe5))

- **document**: Human-readable Document store linked from journal entries ([094f1bc](https://github.com/gitehr/gitehr/commit/094f1bc5f0712a55afc31d67a0cd9ba9c93e3449))

### Refactor

- **mpi**: Rename "Master Patient Index" to "Main Patient Index" ([522a0d2](https://github.com/gitehr/gitehr/commit/522a0d2776ec21929673270e76b1f9ed5a2292b6))

- **calc**: Move clinical calculators out to the gitehr/tools repo ([926ed58](https://github.com/gitehr/gitehr/commit/926ed58c288da2d4745a9ea014dc518a8fa4a78e))

### Styling

- Clear clippy warnings so the workspace passes -D warnings ([157264d](https://github.com/gitehr/gitehr/commit/157264d8aacfcd77e28baa1fdd14cc16752fdca2))

### Tests

- **cli**: Repoint integration tests to current API; drop removed-feature tests ([4ed6cf1](https://github.com/gitehr/gitehr/commit/4ed6cf1f6b0de79f758298e8843294641547f044))

## [0.1.8] - 2026-06-01

### Bug fixes

- Update URL for opening docs in browser to match new port ([e1c0d4f](https://github.com/gitehr/gitehr/commit/e1c0d4f30d64790643e70d5588d624c814d9da5c))

- Update template path and journal tests for workspace structure ([c1c24b7](https://github.com/gitehr/gitehr/commit/c1c24b765786bbefa40961d0b9142c0079e36f5d))

### CI

- Replace MkDocs deploy workflow with Docusaurus ([4d63be0](https://github.com/gitehr/gitehr/commit/4d63be06f3eeea9016a8135563866495a6d21fc5))

### CI / dependencies

- **deps**: Bump clap from 4.5.60 to 4.6.1 ([6d7bb8c](https://github.com/gitehr/gitehr/commit/6d7bb8c08f55459dfc971e84417082d03258d43b))

- **deps**: Bump tracing-subscriber from 0.3.22 to 0.3.23 ([ec9c201](https://github.com/gitehr/gitehr/commit/ec9c20170ef260ffdfc59733a7cec9d43fec06fc))

- **deps**: Bump which from 8.0.0 to 8.0.2 ([be35707](https://github.com/gitehr/gitehr/commit/be35707d3978c4f8f8513e18edaecd319ae5a5bb))

- **deps**: Bump serial_test from 3.3.1 to 3.4.0 ([eda3c03](https://github.com/gitehr/gitehr/commit/eda3c03c4e9ac3d625f0445501f96c0428f112f0))

- **deps**: Bump futures from 0.3.31 to 0.3.32 ([823fbc0](https://github.com/gitehr/gitehr/commit/823fbc088d6b592c7cf055224fe8df583e4d2517))

- **deps**: Bump actions/upload-pages-artifact from 4 to 5 ([813b3f8](https://github.com/gitehr/gitehr/commit/813b3f89d2d2c56d9e7bd9384ddd88ccc06b448b))

- **deps**: Bump actions/configure-pages from 5 to 6 ([eeb71c7](https://github.com/gitehr/gitehr/commit/eeb71c7c0b5682ee12ba464ed0b9edf226b3dddc))

- **deps**: Bump actions/deploy-pages from 4 to 5 ([53545b3](https://github.com/gitehr/gitehr/commit/53545b3d58cb420dcd268fb46068f318db03ca82))

### Chores

- Update docker-compose.yml to change port mapping for docs service ([067a86e](https://github.com/gitehr/gitehr/commit/067a86ebf6f33744177a84ef062df14c83ac9f41))

- Add .marcus/ to gitignore for personal notes ([17b7ca4](https://github.com/gitehr/gitehr/commit/17b7ca4e9606ae1c1a4b3cb3b7e13881315613cc))

### Documentation

- Add FSH and Archie integration strategies to specs ([3a72bd4](https://github.com/gitehr/gitehr/commit/3a72bd41632af05eb8546b8808fb0a691bcaa91c))

- Add comprehensive MCP implementation summary ([ee0c4d3](https://github.com/gitehr/gitehr/commit/ee0c4d374b13493e88ed51517e0e427242cb622e))

- Add MCP usage guide and test script ([2d0c4f3](https://github.com/gitehr/gitehr/commit/2d0c4f3a7c7c03f8ad38ccad860d14cd5ce0af2b))

### Features

- Remove obsolete documentation files and update roadmap and spec for plugin system ([1f1afe8](https://github.com/gitehr/gitehr/commit/1f1afe8548270bdbb91093bda4e95262767abb52))

- Migrate docs site from MkDocs to Docusaurus 3.9.2 ([c5ee2ef](https://github.com/gitehr/gitehr/commit/c5ee2ef1218de2d36a9f2b80e06d7166f01c57eb))

- Add patient management commands to store module ([0f4c270](https://github.com/gitehr/gitehr/commit/0f4c27076c4333577b38010e4d78778f9d4d7711))

- Implement content-addressed storage and timestamp-prefixed UUIDs ([9ecead5](https://github.com/gitehr/gitehr/commit/9ecead56e75f1804ddc4b2d700a1c221a91ae230))

- Integrate Tauri MCP Bridge plugin for GUI development ([c6cd420](https://github.com/gitehr/gitehr/commit/c6cd420b66a8a98153afea7004494ba46ab187c7))

- Implement store root creation for multi-patient repositories ([5d299cf](https://github.com/gitehr/gitehr/commit/5d299cfd9c88c969a4d15a982e92c2b70743a77a))

- Implement MCP server and convert to workspace ([82b2ccf](https://github.com/gitehr/gitehr/commit/82b2ccf6ff8997e6aa60098a9a261774babf243b))

- Add script to update Rust and Node dependencies ([14adb51](https://github.com/gitehr/gitehr/commit/14adb517830e8d4be53d5472c3313d16def14550))

- Add agent and copilot instructions, update agents documentation for CLI commands ([a2f4ad0](https://github.com/gitehr/gitehr/commit/a2f4ad0cc8aad7ab35f846cd6da49c76d28f650e))

- Add .sisyphus/ to .gitignore ([85239e9](https://github.com/gitehr/gitehr/commit/85239e9bac4f32b02096c091e07b4803f2bf38a7))

## [0.1.7] - 2026-01-23

### Bug fixes

- Update .gitignore to ignore all .env files ([efc50e8](https://github.com/gitehr/gitehr/commit/efc50e8777ab253a813164695708cd99d91bc920))

### Chores

- Update dependencies and refactor YAML handling in journal commands ([ed31903](https://github.com/gitehr/gitehr/commit/ed319038b8a916be22127148e2d17e90d3496d95))

### Features

- Enhance App layout with transparent background and updated header/navbar/aside offsets ([f46bf24](https://github.com/gitehr/gitehr/commit/f46bf24527f2a116777c5bb459de3d6ece86ca3c))

- Add initial GitEHR roadmap outlining core CLI features, unimplemented commands, and documentation alignment ([aaece56](https://github.com/gitehr/gitehr/commit/aaece56409809ff73b5879849ec57ef120cd769f))

- Update command documentation and add placeholders for unimplemented features ([b040957](https://github.com/gitehr/gitehr/commit/b0409571deaa0437664782398cf8d328e1e88d0c))

- Add initial Copilot instructions and GitEHR agents documentation ([c30c32e](https://github.com/gitehr/gitehr/commit/c30c32e0939a101aca0d8dcbefb9d062e20d9dde))

## [0.1.6] - 2026-01-23

### Chores

- Update dependencies and remove unused packages in Cargo.lock ([892ceb3](https://github.com/gitehr/gitehr/commit/892ceb37bc2c45e592fc89943a0efc37a6405b3c))

### Features

- Add script to regenerate application icons from SVG source, and update all icons everywhere ([3e06ad9](https://github.com/gitehr/gitehr/commit/3e06ad95a57b6e5d36ec0fe1f440569523f1bb6a))

- Enhance documentation with glossary and improve links in the narrative ([52f16fc](https://github.com/gitehr/gitehr/commit/52f16fc4c1c3ac7f978280cba0207c8a3a552465))

- Update Tauri configuration and add application icon support ([ed52898](https://github.com/gitehr/gitehr/commit/ed52898fa2d5978c255e554c83b77707fcf6544a))

- Update dependencies and enhance UI with Mantine components ([fd1007c](https://github.com/gitehr/gitehr/commit/fd1007c12c26b02f1ce423af02f3fa2982d9f29d))

- Add command documentation for contributor, decrypt, encrypt, state, status, and transport ([96c6b24](https://github.com/gitehr/gitehr/commit/96c6b24612f9fd8f346f50afe59a848611c3fe54))

- Add dependabot configuration for automated dependency updates ([53d41fe](https://github.com/gitehr/gitehr/commit/53d41fed57b2b55dfe8b4ea2136fd28362ea3e88))

- Enhance script functionality and add version bumping script ([2091335](https://github.com/gitehr/gitehr/commit/209133579b66088a74bb755457c163d4ecc24965))

- Initialize Tauri + React application with Vite ([87c3cde](https://github.com/gitehr/gitehr/commit/87c3cde252f11e6a148a3802bbc9c52c4b467636))

## [0.1.5] - 2026-01-23


