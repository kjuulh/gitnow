// gitnow — a TOOL_EXTERNAL component.
//
// The binary is built and released by this repo's own pipeline and hosted on
// git.kjuulh.io; forest only records where to fetch it from and what it should
// hash to. Hence `external:` in forest.cue rather than `upload:` — forest
// neither builds nor stores the artifact.
//
// #Commands, #Hooks and #Spec are deliberately omitted: external manifests have
// no describe protocol, so they cannot carry commands or hooks, and the
// publish-time validator rejects them.
package gitnow

import sdk "forest.sh/forest/sdk@v0"

#Tool: sdk.#ForestTool & {
	name:             "gitnow"
	argv_passthrough: true
	description:      "Navigate, clone, and enter git projects as fast as you can type."
}
