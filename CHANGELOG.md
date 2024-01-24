# Changelog

This project follows [Semantic Versioning]. \
In this case, a "breaking change" is one that alters what commands are passed to
the CLI - as well as how preexisting ones behave - in a way that breaks
previous user inputs.

## Version 2.0.0 - *Upcoming*

This version includes a total rewrite of the codebase and improvements to
all commands.

### Breaking Changes

- The generated `images.slop` file now includes a `!version` keyvalue.
  - The program will now **panic** if this keyvalue is not present or holds the
    wrong number.
  - The current `images.slop` version is now `1`.
- `build` now needs the `sounds.slop` reference file to compare sound files and
  will **panic** otherwise.

#### Migrating From `1.0.0` to `2.0.0`

Simply run the `gen` command again in version [2.0.0].

### Added

- `gen` now generates sound reference files.
- `scan` now compares image sizes in addition to file names.
- `scan` now compares sound files.

## Version 1.0.0 - 2023-08-08

This is the initial release.

<!-- References -->

<!--[1.0.0]: #version-100---2023-08-08-->
[2.0.0]: #version-200---upcoming
[Semantic Versioning]: https://semver.org
