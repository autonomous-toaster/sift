/**
 * sift — pi extension that routes bash and read through sift.
 *
 * - Overrides the `read` tool to use `sift-read` (range-aware caching)
 * - Wraps every `bash` command with `sift -c "..."` (streaming, caching)
 * - Propagates AI_SESSION so cache persists across invocations
 * - Resets cache on compaction/fork/switch/shutdown
 * - Nudges the agent to understand sift cache markers
 *
 * Usage:
 *   pi -e ./integrations/pi/sift.ts
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { createBashTool } from "@earendil-works/pi-coding-agent";
import { execSync } from "child_process";
import { readFileSync } from "fs";
import { access as fsAccess } from "fs/promises";
import { Type } from "typebox";

function shQuote(s: string): string {
	return "'" + s.replace(/'/g, "'\\''") + "'";
}

let currentSessionId = "default";

function siftEnv(sessionId?: string): NodeJS.ProcessEnv {
	return { ...process.env, AI_SESSION: sessionId ?? currentSessionId };
}

function siftExec(cmd: string): string {
	return execSync(`sift -c ${JSON.stringify(cmd)}`, {
		env: siftEnv(),
		encoding: "utf-8",
		maxBuffer: 10 * 1024 * 1024,
	}).toString();
}

const readSchema = Type.Object({
	path: Type.String({ description: "Path to the file to read (relative or absolute)" }),
	offset: Type.Optional(Type.Number({ description: "Line number to start reading from (1-indexed)" })),
	limit: Type.Optional(Type.Number({ description: "Maximum number of lines to read" })),
});

export default function (pi: ExtensionAPI) {
	// ── Track session ID ────────────────────────────────────────────
	pi.on("session_start", (_event, ctx) => {
		currentSessionId = ctx.sessionManager.getSessionId() ?? "default";
	});

	// ── Override read tool ──────────────────────────────────────────
	pi.registerTool({
		name: "read",
		label: "read",
		description:
			"Read the contents of a file. Supports text files and images (jpg, png, gif, webp, bmp). " +
			"Images are sent as attachments. For text files, output is truncated to 2000 lines or 50KB " +
			"(whichever is hit first). Use offset/limit for large files. " +
			"When you need the full file, continue with offset until complete.",
		parameters: readSchema,

		async execute(_toolCallId, params, _signal, _onUpdate, ctx) {
			const { path, offset, limit } = params;

			let cmd = `sift-read ${shQuote(path)}`;
			if (offset !== undefined) {
				cmd += ` ${offset}`;
				if (limit !== undefined) {
					cmd += ` ${limit}`;
				}
			}

			const output = siftExec(cmd);

			// Handle image files
			const absolutePath = path.startsWith("/")
				? path
				: `${ctx?.cwd ?? process.cwd()}/${path}`;
			let mimeType: string | undefined;
			try {
				const mime = await import("mime-types");
				mimeType = mime.lookup(absolutePath) || undefined;
			} catch {
				// mime-types not available
			}

			if (mimeType?.startsWith("image/")) {
				const buffer = readFileSync(absolutePath);
				return {
					content: [
						{ type: "text" as const, text: `Read image file [${mimeType}]` },
						{ type: "image" as const, data: buffer.toString("base64"), mimeType },
					],
					details: {},
				};
			}

			return {
				content: [{ type: "text" as const, text: output }],
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
			env: { ...env, AI_SESSION: currentSessionId },
		}),
	});

	pi.registerTool({
		...bashTool,
		execute: async (id, params, signal, onUpdate, ctx) => {
			return bashTool.execute(id, params, signal, onUpdate, ctx);
		},
	});

	// ── System prompt nudge ─────────────────────────────────────────
	pi.on("before_agent_start", async (event) => {
		return {
			systemPrompt:
				event.systemPrompt +
				'\n\n[sift] caches file reads. When you see "[sift] ... unchanged (cached)", ' +
				'the content is already in this conversation — say "same as before" and move on. ' +
				"Do NOT re-read or bypass the cache unless you have a specific reason to believe " +
				"the file changed on disk.",
		};
	});

	// ── Reset cache on session events ───────────────────────────────
	const resetCache = (sessionId: string) => {
		try {
			execSync("sift -c reset", {
				env: siftEnv(sessionId),
				encoding: "utf-8",
			});
		} catch {
			// Fail-open
		}
	};

	pi.on("session_compact", (_event, ctx) => {
		resetCache(ctx.sessionManager.getSessionId() ?? "default");
	});
	pi.on("session_tree", (_event, ctx) => {
		resetCache(ctx.sessionManager.getSessionId() ?? "default");
	});
	pi.on("session_fork", (_event, ctx) => {
		resetCache(ctx.sessionManager.getSessionId() ?? "default");
	});
	pi.on("session_switch", (_event, ctx) => {
		resetCache(ctx.sessionManager.getSessionId() ?? "default");
	});
	pi.on("session_shutdown", (_event, ctx) => {
		resetCache(ctx.sessionManager.getSessionId() ?? "default");
	});
}
