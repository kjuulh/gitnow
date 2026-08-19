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
		git_url: "https://git.kjuulh.io/kjuulh/gitnow"
		owner:   "kjuulh"
	}
}

forest: component: sdk.#ForestComponent & {
	name:    project.name
	version: "0.5.1"

	// Shell integration, declared rather than pasted into every user's rc file
	// (forest DATA-588). Forest runs `gitnow init zsh` once when the binary is
	// fetched, caches stdout, and serves it from `forest shell zsh` — so
	// `eval "$(forest shell zsh)"` is all a user needs to get `git-now`/`gn`,
	// and nobody pays a cold-cache download at shell startup to get them.
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
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.5.1/gitnow_0.5.1_darwin_arm64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "b07c09038ed568f949fb3675f332556dbf64cf4a55e37c92b3476b64ebcb3fed"
				archive_sha256:    "f0f8125ebcbfd1992ef8afd204f14fc143d1ffdc6076b7e43e71eda8de700cdf"
			},
			{
				os:                "linux"
				arch:              "amd64"
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.5.1/gitnow_0.5.1_linux_amd64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "940398796dda34a74e4e9c464b90fc86a9f22a57e3e908a09a88a4e7f0f008fe"
				archive_sha256:    "1d84586bc663c91e00c411165b6f8fe520588209735943e9364edae23703a3fb"
			},
			{
				os:                "linux"
				arch:              "arm64"
				url:               "https://git.kjuulh.io/kjuulh/gitnow/releases/download/v0.5.1/gitnow_0.5.1_linux_arm64.tar.gz"
				archive:           "tar.gz"
				binary_in_archive: "gitnow"
				sha256:            "e82bf0706d0452d632f173298b51e4e842ccb7a9e4bc4ab7080007b6edbcfa6c"
				archive_sha256:    "b9d0423427b953995209978ad2f11f6c2dd86613fbb951eaae00743ebeba322f"
			},
		]
	}
}
