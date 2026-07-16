/**
 * sift — pi extension that routes bash and read through sift.
 *
 * - Overrides the `read` tool to use `sift-read` (range-aware caching)
 * - Wraps every `bash` command with `sift -c "..."` (streaming, caching)
 * - Propagates AI_SESSION so cache persists across invocations
 * - Resets cache on compaction/fork/switch/shutdown
 *
 * Usage:
 *   pi -e ./integrations/pi/sift.ts
 */

import type { ExtensionAPI, BashToolInput } from "@earendil-works/pi-coding-agent";
import { createBashTool, isToolCallEventType } from "@earendil-works/pi-coding-agent";
import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";
import { Type } from "typebox";

function getSessionId(ctx?: { sessionManager?: { getSessionId?: () => string } }): string {
	return ctx?.sessionManager?.getSessionId?.() ?? "default";
}

function siftEnv(sessionId: string): NodeJS.ProcessEnv {
	return { ...process.env, AI_SESSION: sessionId };
}

function siftExec(cmd: string, sessionId: string): string {
	try {
		return execSync(`sift -c ${JSON.stringify(cmd)}`, {
			env: siftEnv(sessionId),
			encoding: "utf-8",
			maxBuffer: 10 * 1024 * 1024, // 10MB
		});
	} catch (e: any) {
		return e.stdout ?? "";
	}
}

const readSchema = Type.Object({
	path: Type.String({ description: "Path to the file to read (relative or absolute)" }),
	offset: Type.Optional(Type.Number({ description: "Line number to start reading from (1-indexed)" })),
	limit: Type.Optional(Type.Number({ description: "Maximum number of lines to read" })),
	bypass_cache: Type.Optional(
		Type.Boolean({
			description:
				"If true, bypass sift-read cache and return fresh content for the requested scope",
		}),
	),
});

export default function (pi: ExtensionAPI) {
	// ── Override read tool ──────────────────────────────────────────
	pi.registerTool({
		name: "read",
		label: "read",
		description:
			"Read the contents of a file. Supports text files and images (jpg, png, gif, webp). " +
			"Images are sent as attachments. For text files, output is truncated to 2000 lines or 50KB. " +
			"Use offset/limit for large files. Returns full text or unchanged marker. " +
			"Set bypass_cache=true to force baseline output.",
		parameters: readSchema,

		async execute(_toolCallId, params, _signal, _onUpdate, ctx) {
			const sessionId = getSessionId(ctx);
			const { path, offset, limit, bypass_cache } = params;

			// Build sift-read command
			let cmd = "sift-read";
			if (bypass_cache) {
				cmd = cmd .. " --fresh";
			}
			cmd = cmd .. " " .. JSON.stringify(path);
			if (offset !== undefined) {
				cmd = cmd .. " " .. offset;
				if (limit !== undefined) {
					cmd = cmd .. " " .. limit;
				}
			}

			const output = siftExec(cmd, sessionId);

			// Check for unchanged marker
			if (output:match("^%[sift%]")) {
				return {
					content: [{ type: "text" as const, text: output }],
					details: { cached: true },
				};
			}

			// Truncate to 50KB
			const maxBytes = 50 * 1024;
			let text = output;
			if (Buffer.byteLength(text, "utf-8") > maxBytes) {
				text = text.slice(0, maxBytes) .. "\n\n[Output truncated at 50KB]";
			}

			return {
				content: [{ type: "text" as const, text }],
				details: {},
			};
		},
	});

	// ── Wrap bash with sift ──────────────────────────────────────────
	const cwd = process.cwd();
	const bashTool = createBashTool(cwd, {
		spawnHook: ({ command, cwd, env }) => ({
			command: `sift -c ${JSON.stringify(command)}`,
			cwd,
			env: { ...env, AI_SESSION: getSessionId() },
		}),
	});

	pi.registerTool({
		...bashTool,
		execute: async (id, params, signal, onUpdate, ctx) => {
			return bashTool.execute(id, params, signal, onUpdate, ctx);
		},
	});

	// ── Reset cache on session events ───────────────────────────────
	const resetCache = () => {
		try {
			execSync("sift -c reset", {
				env: siftEnv(getSessionId()),
				encoding: "utf-8",
			});
		} catch {
			// Fail-open
		}
	};

	pi.on("session_compact", resetCache);
	pi.on("session_tree", resetCache);
	pi.on("session_fork", resetCache);
	pi.on("session_switch", resetCache);
	pi.on("session_shutdown", resetCache);
}
