<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# gitehr-clincalc

`gitehr-clincalc` is the external GitEHR plugin for [clincalc](https://github.com/pacharanero/clincalc). It exposes clincalc's calculator CLI as `gitehr clincalc` and can record a verified calculation in the current GitEHR repository.

Install it with Cargo:

```console
$ cargo install gitehr-clincalc --locked
```

The installed `gitehr-clincalc` binary must be on `PATH`. GitEHR discovers it automatically:

```console
$ gitehr clincalc feverpain --input '{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}'
$ gitehr clincalc record feverpain --input '{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}'
```

The plugin deliberately contains no calculator algorithms or GitEHR repository implementation. It calls clincalc's public calculator API, then asks the public `gitehr journal add` command to create the immutable record.

Code is licensed under AGPL-3.0-or-later. Documentation is licensed under CC-BY-SA-4.0. Calculator algorithms and their evidence or distribution licences are owned and declared by clincalc.
