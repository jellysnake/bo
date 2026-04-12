// check-rust.ts — project-local pi extension
//
// Runs per-file Rust checks after every edit/write to a .rs file and feeds
// failures back into the LLM context as errors so the agent self-corrects
// before the next turn.
//
// Checks (in order):
//   1. rustfmt --check <file>   — formatting for the changed file only
//   2. cargo clippy (JSON mode) — clippy diagnostics filtered to the changed file

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { resolve, relative } from "node:path";

function stripAnsi(s: string): string {
  return s.replace(/\x1b\[[0-9;]*[A-Za-z]/g, "");
}

export default function (pi: ExtensionAPI) {
  pi.on("tool_result", async (event, ctx) => {
    if (event.toolName !== "edit" && event.toolName !== "write") return;

    const rawPath: string | undefined = (event.input as any)?.path;
    if (!rawPath?.endsWith(".rs")) return;

    const absolutePath = resolve(ctx.cwd, rawPath);
    const relPath = relative(ctx.cwd, absolutePath); // e.g. "src/main.rs"

    const errors: string[] = [];

    // ── 1. rustfmt --check <file> ────────────────────────────────────────────
    const fmt = await pi.exec("rustfmt", ["--check", absolutePath], {
      cwd: ctx.cwd,
      signal: ctx.signal,
    });
    if (fmt.code !== 0) {
      errors.push("rustfmt: formatting differs — run `cargo fmt` to fix");
    }

    // ── 2. cargo clippy filtered to this file ────────────────────────────────
    // --message-format=json writes one JSON object per line to stdout.
    // We parse each line and keep only compiler-message entries whose spans
    // touch the file we just edited, avoiding noise from unrelated files.
    const clippy = await pi.exec(
      "cargo",
      ["clippy", "--message-format=json", "--", "-D", "warnings"],
      { cwd: ctx.cwd, signal: ctx.signal, timeout: 60_000 },
    );

    const diagnostics = clippy.stdout
      .split("\n")
      .filter((line) => line.trim().startsWith("{"))
      .flatMap((line) => {
        try {
          return [JSON.parse(line)];
        } catch {
          return [];
        }
      })
      .filter(
        (msg: any) =>
          msg.reason === "compiler-message" &&
          (msg.message?.spans ?? []).some((s: any) => s.file_name === relPath),
      );

    if (diagnostics.length > 0) {
      const rendered = diagnostics
        .map((d: any) => stripAnsi(d.message?.rendered ?? ""))
        .filter(Boolean)
        .join("\n");
      errors.push(`cargo clippy:\n${rendered}`);
    }

    if (errors.length === 0) return;

    return {
      content: [
        ...event.content,
        { type: "text", text: `\n---\nRust checks failed:\n\n${errors.join("\n\n")}` },
      ],
      isError: true,
    };
  });
}
