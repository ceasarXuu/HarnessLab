const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { spawnAttached, run } = require("./common");
const { sourceDir } = require("./state");
const { ensureSource } = require("./source");
const { ensureReady } = require("./bootstrap");

const backendHost = process.env.ORNNLAB_BACKEND_HOST || "127.0.0.1";
const backendPort = process.env.ORNNLAB_BACKEND_PORT || "8765";
const frontendHost = process.env.ORNNLAB_FRONTEND_HOST || "127.0.0.1";
const frontendPort = process.env.ORNNLAB_FRONTEND_PORT || "5173";
const startupTimeoutMs = readStartupTimeoutMs(process.env.ORNNLAB_STARTUP_TIMEOUT_SECONDS);
const livePath = "/api/webui/v1/system/live";
const localhostNoProxyEntries = ["127.0.0.1", "localhost", "::1"];

ensureLocalhostNoProxy(process.env);

function runBackend(args) {
  ensureSource();
  const backend = backendInvocation(["web", "--host", backendHost, "--port", backendPort, ...args]);
  run(backend.command, backend.args, { cwd: sourceDir });
}

function runDoctor(args) {
  ensureSource();
  run("uv", ["run", "ornnlab", "doctor", ...args], { cwd: sourceDir });
}

function runFrontend(args) {
  ensureSource();
  run(
    "npm",
    ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort", ...args],
    { cwd: path.join(sourceDir, "frontend"), env: frontendEnvironment() },
  );
}

function printLaunchUrls() {
  console.log("");
  console.log("OrnnLab backend is starting.");
  console.log(`Frontend: http://${frontendHost}:${frontendPort}/`);
  console.log(`Backend API: http://${backendHost}:${backendPort}/`);
  console.log("Press Ctrl+C to stop both servers.");
  console.log("");
}

async function runDev({ setupIfMissing = false } = {}) {
  if (setupIfMissing) {
    await ensureReady();
  } else {
    ensureSource();
  }
  const backendUrl = `http://${backendHost}:${backendPort}`;
  const frontendUrl = `http://${frontendHost}:${frontendPort}`;
  const backendCommand = backendInvocation(["web", "--host", backendHost, "--port", backendPort]);
  const backend = spawnAttached(backendCommand.command, backendCommand.args, {
    cwd: sourceDir,
    detached: process.platform !== "win32",
  });
  let frontend;
  let stopping = false;
  const children = [backend];
  const shutdown = () => {
    if (stopping) return;
    stopping = true;
    for (const child of children) {
      if (child && child.exitCode === null) terminateServiceTree(child);
    }
  };
  const onSignal = () => shutdown();
  process.once("SIGINT", onSignal);
  process.once("SIGTERM", onSignal);

  try {
    await waitForHealth(`${backendUrl}${livePath}`, "backend", backend);
    frontend = spawnAttached(
      "npm",
      ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort"],
      {
        cwd: path.join(sourceDir, "frontend"),
        detached: process.platform !== "win32",
        env: frontendEnvironment(backendUrl),
      },
    );
    children.push(frontend);
    await waitForHealth(`${frontendUrl}${livePath}`, "frontend API proxy", frontend);
    printLaunchUrls();
    await waitForServiceExit(children, () => stopping);
  } finally {
    shutdown();
    await Promise.all(children.map(waitForChildExit));
    process.off("SIGINT", onSignal);
    process.off("SIGTERM", onSignal);
  }
}

function frontendEnvironment(apiTarget = `http://${backendHost}:${backendPort}`) {
  const dataMode = process.env.VITE_ORNNLAB_DATA_MODE || "api";
  if (dataMode !== "api" && dataMode !== "mock") {
    throw new Error(`VITE_ORNNLAB_DATA_MODE must be "api" or "mock", received "${dataMode}".`);
  }
  return {
    ...process.env,
    ORNNLAB_API_TARGET: apiTarget,
    ORNNLAB_FRONTEND_PORT: frontendPort,
    VITE_ORNNLAB_DATA_MODE: dataMode,
  };
}

function ensureLocalhostNoProxy(env) {
  env.NO_PROXY = mergeNoProxy(env.NO_PROXY);
  env.no_proxy = mergeNoProxy(env.no_proxy);
  return env;
}

function mergeNoProxy(value) {
  const entries = String(value || "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
  for (const entry of localhostNoProxyEntries) {
    if (!entries.includes(entry)) entries.push(entry);
  }
  return entries.join(",");
}

function readStartupTimeoutMs(value) {
  if (value === undefined || value === "") return 300000;
  const seconds = Number(value);
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 300) {
    throw new Error("ORNNLAB_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 300.");
  }
  return seconds * 1000;
}

function backendInvocation(args = []) {
  const localScript = localOrnnlabScript();
  if (localScript) return { command: localScript, args };
  return { command: "uv", args: ["run", "ornnlab", ...args] };
}

function localOrnnlabScript() {
  const relative = process.platform === "win32"
    ? [".venv", "Scripts", "ornnlab.exe"]
    : [".venv", "bin", "ornnlab"];
  const candidate = path.join(sourceDir, ...relative);
  return fs.existsSync(candidate) ? candidate : null;
}

async function waitForHealth(url, service, child) {
  const deadline = Date.now() + startupTimeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`${service} exited before becoming ready.`);
    }
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // The process may still be binding its local port.
    }
    await sleep(250);
  }
  throw new Error(`${service} did not become ready within ${startupTimeoutMs / 1000}s.`);
}

function waitForServiceExit(children, isStopping) {
  return new Promise((resolve, reject) => {
    const onExit = (name) => (code, signal) => {
      if (isStopping()) {
        resolve();
        return;
      }
      reject(new Error(`${name} exited unexpectedly (${signal || code || "unknown"}).`));
    };
    children.forEach((child, index) => child.once("exit", onExit(index === 0 ? "backend" : "frontend")));
  });
}

function waitForChildExit(child) {
  if (child.exitCode !== null) return Promise.resolve();
  return new Promise((resolve) => child.once("exit", resolve));
}

function terminateServiceTree(child) {
  if (process.platform === "win32") {
    const result = spawnSync("taskkill", ["/pid", String(child.pid), "/t", "/f"], { stdio: "ignore" });
    if (result.error || result.status !== 0) {
      const reason = result.error?.message || `exit status ${result.status ?? "unknown"}`;
      console.warn(`taskkill could not terminate child process tree ${child.pid}: ${reason}`);
      child.kill("SIGTERM");
    }
    return;
  }
  try {
    process.kill(-child.pid, "SIGTERM");
  } catch {
    child.kill("SIGTERM");
  }
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

module.exports = {
  backendHost,
  backendPort,
  frontendHost,
  frontendPort,
  backendInvocation,
  ensureLocalhostNoProxy,
  frontendEnvironment,
  readStartupTimeoutMs,
  runBackend,
  runDoctor,
  runFrontend,
  printLaunchUrls,
  runDev,
};
