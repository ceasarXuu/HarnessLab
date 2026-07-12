#!/usr/bin/env node

const { spawn } = require("node:child_process");

const parsed = parseArgs(process.argv.slice(2));

if (!parsed.command.length || !parsed.token) {
  console.error("missing child wrapper token or command");
  process.exit(2);
}

const child = spawn(parsed.command[0], parsed.command.slice(1), {
  cwd: process.cwd(),
  env: process.env,
  shell: process.platform === "win32",
  stdio: "inherit",
});

let stopping = false;

function forward(signal) {
  if (stopping) return;
  stopping = true;
  try {
    if (process.platform === "win32") child.kill(signal);
    else process.kill(child.pid, signal);
  } catch {
    // child may already be gone
  }
  setTimeout(() => {
    try {
      if (child.exitCode === null) child.kill("SIGKILL");
    } catch {
      // child may already be gone
    }
  }, 2500).unref();
}

process.once("SIGINT", () => forward("SIGINT"));
process.once("SIGTERM", () => forward("SIGTERM"));

child.once("exit", (code, signal) => {
  if (signal) process.exit(signal === "SIGTERM" || signal === "SIGINT" ? 0 : 1);
  process.exit(code ?? 1);
});

function parseArgs(args) {
  const separator = args.indexOf("--");
  const head = separator === -1 ? args : args.slice(0, separator);
  const command = separator === -1 ? [] : args.slice(separator + 1);
  const result = { token: null, command };
  for (let index = 0; index < head.length; index += 1) {
    if (head[index] === "--token") result.token = head[index + 1] || null;
  }
  return result;
}
