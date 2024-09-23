# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2024-09-23

### Other
- add docs

main@origin

- add license
- update to gitea-client
- add publish
- *(release)* 0.2.1

## [0.2.1] - 2024-09-23

### Added
- use termwiz as backend as that enables a ptty, which can be cleaned up nicely
- add errout for interactive for script support and atty for clean output
- add clone spinner
- add spinner around download
- spawn a subshell for session
- implement git clone
- include vhs demo
- add interactive search
- implement naive fuzzy matcher

### Fixed
- *(deps)* update tokio-prost monorepo to v0.13.3
- *(deps)* update rust crate bytes to v1.7.2

### Other
- update gif to include spinner
- clean up ui
- build in cuddle instead of vhs
- build first then run
- clear screen after build
- fix warnings
- update theme for vhs
- *(deps)* update rust crate clap to v4.5.18
- *(deps)* update rust crate pretty_assertions to v1.4.1
- refactor fuzzy match into own function
- cleanup warnings
- move fuzzy search out of command
- refactor/matcher move to a separate file

- move fuzzy search out of command
- Actually add fuzzy matcher

- extract matcher
- update dependencies
- *(deps)* update rust crate anyhow to v1.0.89

## [0.2.0] - 2024-09-14

### Added
- add cache get
- send out wait
- add cache
- add settings config
- add github fetch prs refactoring
- gitea able to pull repositories
- add config

### Docs
- add readme

### Fixed
- don't have to use user for basic auth

### Other
- removed unused code
- move projects list into separate file
- separate files
- move config out
- remove unused libraries

## [0.1.0] - 2024-09-12

### Added
- init

### Docs
- test
