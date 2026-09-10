// Forest project manifest for gitnow.
//
// gitnow is distributed as a TOOL_EXTERNAL component: the release pipeline in
// this repo builds the binaries and publishes them to git.kjuulh.io, and forest
// only records where to fetch each platform from and what it must hash to.
// Nothing is uploaded to the Forest registry — hence `external:` rather than
// `upload:`.
//
// Until now this declaration lived nowhere in the repo; the component was
// published ad hoc, which made it impossible to change without re-deriving it
// by hand. It lives here now so a release and its registry entry can move
// together.
//
// Publishing a new version, after the repo's own release has produced the
// tarballs:
//
//	forest context use understory-prod
//	# for each platform, to fill in the hashes below:
//	forest tool hash <url> --archive tar.gz --binary-in-archive gitnow
//	forest publish
//
// The `sha256` values are the *extracted binary*; `archive_sha256` is the
// tarball, and matches the `checksums.txt` published alongside the release.
//
// Consume:
//	forest global add understory/gitnow
package gitnow

import sdk "forest.sh/forest/sdk@v0"

project: sdk.#ForestProject & {
	name:         "gitnow"
	organisation: "understory"
	description:  "Navigate, clone, and enter git projects as fast as you can type."
	metadata: {
		git_url: "https://github.com/understory-io/gitnow"
		owner:   "understory-io"
	}
}

forest: component: sdk.#ForestComponent & {
	name:    project.name
	version: "0.7.0"

	// Shell integration, declared rather than pasted into every user's rc file
	// (forest DATA-588). Forest runs `gitnow init zsh` once when the binary is
	// fetched, caches stdout, and serves it from `forest shell zsh` — so
	// `eval "$(forest shell zsh)"` is all a user needs to get
	// `git-now`/`gn`/`gi`, and nobody pays a cold-cache download at shell
	// startup to get them.
	//
	// zsh only: `gitnow init` has no bash or fish subcommand. Declaring a shell
	// the tool cannot emit would just cache a failed capture.
	include: shell: init: {
		zsh: ["init", "zsh"]
	}

	// No darwin/amd64: the release pipeline does not build it, and declaring a
	// platform with no artifact would fail at fetch time rather than at publish.
	external: sdk.#ForestExternal & {
		platforms: [
			{
				os:                "macos"
				arch:              "arm64"
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.7.0/gitnow_0.7.0_darwin_arm64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "cba04b3d096ad81055a4bc5a91dcbc5a82465ec77b2163bf1835ac7b95072c7f"
				archive_sha256:    "7d9db7af9454abaccf4dcefc7ce10561d2c8821c657c694c4267957b0794f2aa"
			},
			{
				os:                "linux"
				arch:              "amd64"
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.7.0/gitnow_0.7.0_linux_amd64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "993663d3d9cc2f0da6de72ce4ac23ac68681c1b480c0283aaf3eeab32a44a653"
				archive_sha256:    "03dc75c7197dc5c356bae97db93791681284167811a1e995592a3f903cbc3acd"
			},
			{
				os:                "linux"
				arch:              "arm64"
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.7.0/gitnow_0.7.0_linux_arm64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "698cd7e455c37283bb53b8fc60306010950c58b21464412eb43aa1f4ff0a1ecd"
				archive_sha256:    "81e36b1e65b1171cdd730a5fad9710de4c3d67897a772c4da7131e1a2406199c"
			},
		]
	}
}
