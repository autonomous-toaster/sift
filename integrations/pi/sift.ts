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
import { createBashTool, createReadTool } from "@earendil-works/pi-coding-agent";
import { execSync } from "child_process";
import { access as fsAccess } from "fs/promises";

function shQuote(s: string): string {
	return "'" + s.replace(/'/g, "'\\''") + "'";
}

let currentSessionId = "default";

function siftEnv(sessionId?: string): NodeJS.ProcessEnv {
	return { ...process.env, AI_SESSION: sessionId ?? currentSessionId };
}

function siftExec(cmd: string): string {
	return execSync(`sift -c ${shQuote(cmd)}`, {
		env: siftEnv(),
		encoding: "utf-8",
		maxBuffer: 10 * 1024 * 1024,
	}).toString();
}

export default function (pi: ExtensionAPI) {
	// ── Track session ID ────────────────────────────────────────────
	pi.on("session_start", (_event, ctx) => {
		currentSessionId = ctx.sessionManager.getSessionId() ?? "default";
	});

	// ── Override read tool ──────────────────────────────────────────
	const cwd = process.cwd();
	const readTool = createReadTool(cwd, {
		operations: {
			readFile: async (absolutePath: string) => {
				const result = siftExec(`sift-read ${shQuote(absolutePath)}`);
				return Buffer.from(result);
			},
			access: async (absolutePath: string) => {
				await fsAccess(absolutePath);
			},
		},
	});

	pi.registerTool(readTool);

	// ── Wrap bash with sift ──────────────────────────────────────────
	const bashTool = createBashTool(cwd, {
		spawnHook: ({ command, cwd, env }) => ({
			command: `sift -c ${shQuote(command)}`,
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
