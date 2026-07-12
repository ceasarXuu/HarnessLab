const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { backendHost, backendPort, frontendEnvironment, frontendHost, frontendPort, readStartupTimeoutMs } = require("./dev");
const { launcherDir, sourceDir } = require("./state");
const { ensureSource } = require("./source");

const serviceId = "ornnlab-dev-service";
const startupTimeoutMs = readStartupTimeoutMs(process.env.ORNNLAB_STARTUP_TIMEOUT_SECONDS);
const restartDelaysMs = [1000, 2000, 5000, 10000, 30000];

function devServicePaths() {
  const root = process.env.ORNNLAB_DEV_SERVICE_HOME || path.join(path.dirname(launcherDir), "dev-service");
  return {
    root,
    state: path.join(root, "state.json"),
    logs: {
      daemon: path.join(root, "logs", "daemon.log"),
      backend: path.join(root, "logs", "backend.log"),
      frontend: path.join(root, "logs", "frontend.log"),
    },
  };
}

async function handleDevCommand(args = []) {
  const [subcommand = "foreground", ...rest] = args;
  if (subcommand === "start") return startDaemon();
  if (subcommand === "stop") return stopDaemon();
  if (subcommand === "restart") {
    await stopDaemon({ quiet: true });
    return startDaemon();
  }
  if (subcommand === "status") return printStatus(rest.includes("--json"));
  if (subcommand === "logs") return printLogs(rest);
  if (subcommand === "_daemon") return runDaemon();
  return null;
}

async function startDaemon() {
  ensureSource();
  ensureServiceHome();
  const state = readState();
  if (isPidAlive(state.daemonPid)) {
    const current = await currentStatus();
    console.log(`OrnnLab dev service is ${current.status}.`);
    console.log(`Frontend: ${current.frontendUrl}`);
    console.log(`Backend API: ${current.backendUrl}`);
    return;
  }
  const daemonLog = fs.openSync(devServicePaths().logs.daemon, "a");
  const child = spawn(process.execPath, [path.join(__dirname, "..", "bin", "ornnlab.js"), "dev", "_daemon"], {
    cwd: path.join(__dirname, ".."),
    detached: true,
    env: { ...process.env, ORNNLAB_DEV_DAEMON_CHILD: "1" },
    stdio: ["ignore", daemonLog, daemonLog],
  });
  child.unref();
  fs.closeSync(daemonLog);
  await waitForDaemonReady();
  const ready = await currentStatus();
  if (ready.status === "error") {
    throw new Error(ready.lastError || "OrnnLab dev service failed to start.");
  }
  console.log("OrnnLab dev service is running.");
  console.log(`Frontend: ${ready.frontendUrl}`);
  console.log(`Backend API: ${ready.backendUrl}`);
}

async function stopDaemon({ quiet = false } = {}) {
  const state = readState();
  if (!isPidAlive(state.daemonPid)) {
    await stopRecordedChildren(state);
    writeState({ status: "stopped", stoppedAt: new Date().toISOString(), desiredState: "stopped" });
    if (!quiet) console.log("OrnnLab dev service is stopped.");
    return;
  }
  try {
    process.kill(state.daemonPid, "SIGTERM");
  } catch {
    await stopRecordedChildren(state);
  }
  await waitUntilStopped();
  if (!quiet) console.log("OrnnLab dev service is stopped.");
}

async function printStatus(json = false) {
  const status = await currentStatus();
  if (json) {
    console.log(JSON.stringify(status, null, 2));
    return;
  }
  console.log(`Status: ${status.status}`);
  console.log(`Frontend: ${status.frontendUrl}`);
  console.log(`Backend API: ${status.backendUrl}`);
  console.log(`Daemon PID: ${status.daemonPid || "-"}`);
  if (status.lastError) console.log(`Last error: ${status.lastError}`);
}

function printLogs(args = []) {
  const paths = devServicePaths();
  const target = args.includes("--backend") ? paths.logs.backend : args.includes("--frontend") ? paths.logs.frontend : paths.logs.daemon;
  if (!fs.existsSync(target)) return;
  process.stdout.write(fs.readFileSync(target, "utf8"));
}

async function runDaemon() {
  ensureSource();
  ensureServiceHome();
  const backendUrl = `http://${backendHost}:${backendPort}`;
  const frontendUrl = `http://${frontendHost}:${frontendPort}`;
  const runtime = { backend: null, frontend: null, stopping: false, restartAttempts: { backend: 0, frontend: 0 } };
  writeState(baseState("starting", { backendUrl, frontendUrl, daemonPid: process.pid, desiredState: "running" }));
  logEvent("dev_service.start_requested", { backendUrl, frontendUrl });

  const shutdown = async () => {
    if (runtime.stopping) return;
    runtime.stopping = true;
    writeState({ ...readState(), status: "stopping", desiredState: "stopped" });
    await stopChild(runtime.frontend);
    await stopChild(runtime.backend);
    writeState({ ...baseState("stopped", { backendUrl, frontendUrl }), stoppedAt: new Date().toISOString(), desiredState: "stopped" });
    logEvent("dev_service.stopped", { reason: "requested" });
    process.exit(0);
  };
  process.once("SIGINT", shutdown);
  process.once("SIGTERM", shutdown);

  try {
    runtime.backend = await startManagedChild("backend", backendUrl, runtime);
    runtime.frontend = await startManagedChild("frontend", `${frontendUrl}/api/webui/v1/system/health`, runtime);
    markRunning(runtime, backendUrl, frontendUrl);
    logEvent("dev_service.started", { backendPid: runtime.backend.pid, frontendPid: runtime.frontend.pid });
  } catch (error) {
    runtime.stopping = true;
    logEvent("dev_service.restart_gave_up", { reason: error.message }, "error");
    await stopChild(runtime.frontend);
    await stopChild(runtime.backend);
    writeState({ ...readState(), status: "error", lastError: error.message });
    process.exit(1);
  }
  setInterval(() => {}, 60_000);
}

async function startManagedChild(name, healthUrl, runtime) {
  const paths = devServicePaths();
  const logPath = name === "backend" ? paths.logs.backend : paths.logs.frontend;
  const { command, args, cwd, env } = commandFor(name);
  const logFd = fs.openSync(logPath, "a");
  const child = spawn(command, args, {
    cwd,
    detached: process.platform !== "win32",
    env,
    shell: process.platform === "win32",
    stdio: ["ignore", logFd, logFd],
  });
  fs.closeSync(logFd);
  await waitForHealth(healthUrl, name, child);
  child.once("exit", (code, signal) => void onChildExit(name, code, signal, runtime));
  return child;
}

async function onChildExit(name, code, signal, runtime) {
  if (runtime.stopping) return;
  logEvent("dev_service.child_exited", { child: name, code, signal }, "warn");
  const attempts = runtime.restartAttempts[name] + 1;
  runtime.restartAttempts[name] = attempts;
  if (attempts > restartDelaysMs.length) {
    writeState({ ...readState(), status: "error", lastError: `${name} exceeded restart limit` });
    logEvent("dev_service.restart_gave_up", { child: name, attempts }, "error");
    return;
  }
  const delay = restartDelaysMs[attempts - 1];
  writeState({ ...readState(), status: "degraded", lastError: `${name} exited` });
  logEvent("dev_service.restart_scheduled", { child: name, attempt: attempts, delayMs: delay }, "warn");
  await sleep(delay);
  if (runtime.stopping) return;
  const backendUrl = `http://${backendHost}:${backendPort}`;
  const frontendUrl = `http://${frontendHost}:${frontendPort}`;
  const healthUrl = name === "backend" ? backendUrl : `${frontendUrl}/api/webui/v1/system/health`;
  runtime[name] = await startManagedChild(name, healthUrl, runtime);
  markRunning(runtime, backendUrl, frontendUrl);
}

function commandFor(name) {
  const override = name === "backend" ? process.env.ORNNLAB_DEV_BACKEND_COMMAND : process.env.ORNNLAB_DEV_FRONTEND_COMMAND;
  if (override) {
    const [command, ...args] = splitCommand(override);
    return { command, args, cwd: sourceDir, env: process.env };
  }
  if (name === "backend") {
    return {
      command: "uv",
      args: ["run", "ornnlab", "web", "--host", backendHost, "--port", backendPort],
      cwd: sourceDir,
      env: { ...process.env, ORNNLAB_RESTART_COMMAND: `${process.execPath} ${path.join(__dirname, "..", "bin", "ornnlab.js")} dev restart` },
    };
  }
  return {
    command: "npm",
    args: ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort"],
    cwd: path.join(sourceDir, "frontend"),
    env: frontendEnvironment(`http://${backendHost}:${backendPort}`),
  };
}

function splitCommand(value) {
  return value.match(/(?:[^\s"]+|"[^"]*")+/g)?.map((part) => part.replace(/^"|"$/g, "")) || [];
}

function markRunning(runtime, backendUrl, frontendUrl) {
  runtime.restartAttempts.backend = 0;
  runtime.restartAttempts.frontend = 0;
  writeState({
    ...baseState("running", { backendUrl, frontendUrl, daemonPid: process.pid }),
    backendPid: runtime.backend?.pid,
    frontendPid: runtime.frontend?.pid,
  });
}

function baseState(status, values = {}) {
  return {
    serviceId,
    status,
    backendUrl: `http://${backendHost}:${backendPort}`,
    frontendUrl: `http://${frontendHost}:${frontendPort}`,
    dataMode: process.env.VITE_ORNNLAB_DATA_MODE || "api",
    updatedAt: new Date().toISOString(),
    ...values,
  };
}

function ensureServiceHome() {
  const paths = devServicePaths();
  fs.mkdirSync(path.dirname(paths.state), { recursive: true });
  fs.mkdirSync(path.dirname(paths.logs.daemon), { recursive: true });
}

function readState() {
  try {
    return JSON.parse(fs.readFileSync(devServicePaths().state, "utf8"));
  } catch {
    return baseState("stopped", { desiredState: "stopped" });
  }
}

function writeState(state) {
  ensureServiceHome();
  fs.writeFileSync(devServicePaths().state, `${JSON.stringify(state, null, 2)}\n`);
}

function logEvent(event, fields = {}, level = "info") {
  ensureServiceHome();
  fs.appendFileSync(devServicePaths().logs.daemon, `${JSON.stringify({ event, level, serviceId, time: new Date().toISOString(), ...fields })}\n`);
}

async function currentStatus() {
  const state = readState();
  const daemonAlive = isPidAlive(state.daemonPid);
  const status = daemonAlive ? state.status : "stopped";
  return { ...state, status, daemonAlive, backendAlive: isPidAlive(state.backendPid), frontendAlive: isPidAlive(state.frontendPid) };
}

function isPidAlive(pid) {
  if (!pid) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

async function waitForDaemonReady() {
  const deadline = Date.now() + startupTimeoutMs;
  while (Date.now() < deadline) {
    const status = await currentStatus();
    if (status.status === "running" || status.status === "error") return;
    await sleep(250);
  }
  throw new Error("OrnnLab dev service did not become ready.");
}

async function waitUntilStopped() {
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    if (!(await currentStatus()).daemonAlive) return;
    await sleep(100);
  }
  throw new Error("OrnnLab dev service did not stop within 10s.");
}

async function waitForHealth(url, service, child) {
  const endpoint = service === "backend" ? `${url}/api/webui/v1/system/health` : url;
  const deadline = Date.now() + startupTimeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`${service} exited before becoming ready`);
    try {
      if ((await fetch(endpoint)).ok) return;
    } catch {
      // local service may still be binding
    }
    await sleep(250);
  }
  throw new Error(`${service} did not become ready`);
}

async function stopRecordedChildren(state) {
  await Promise.all([stopPid(state.frontendPid), stopPid(state.backendPid)]);
}

async function stopChild(child) {
  if (!child || child.exitCode !== null) return;
  await stopPid(child.pid);
}

async function stopPid(pid) {
  if (!isPidAlive(pid)) return;
  if (process.platform === "win32") spawnSync("taskkill", ["/pid", String(pid), "/t", "/f"], { stdio: "ignore" });
  else {
    try {
      process.kill(-pid, "SIGTERM");
    } catch {
      process.kill(pid, "SIGTERM");
    }
  }
  await sleep(250);
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

module.exports = {
  devServicePaths,
  commandFor,
  handleDevCommand,
  isPidAlive,
  readState,
  startDaemon,
  stopDaemon,
};
