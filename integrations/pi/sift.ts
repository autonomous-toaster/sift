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

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { createBashTool } from "@earendil-works/pi-coding-agent";
import { execSync } from "child_process";
import { readFileSync } from "fs";
import { resolve } from "path";
import { Type } from "typebox";

function siftEnv(sessionId: string): NodeJS.ProcessEnv {
	return { ...process.env, AI_SESSION: sessionId };
}

function siftExec(cmd: string, sessionId: string): string {
	return execSync(`sift -c ${JSON.stringify(cmd)}`, {
		env: siftEnv(sessionId),
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
			const sessionId = ctx?.sessionManager?.getSessionId?.() ?? "default";
			const { path, offset, limit } = params;

			let cmd = "sift-read " + JSON.stringify(path);
			if (offset !== undefined) {
				cmd += " " + offset;
				if (limit !== undefined) {
					cmd += " " + limit;
				}
			}

			const output = siftExec(cmd, sessionId);

			// Read the actual file for image detection
			const absolutePath = resolve(ctx?.cwd ?? process.cwd(), path);
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
			env: { ...env, AI_SESSION: env.AI_SESSION ?? "default" },
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
				env: siftEnv("default"),
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
