const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { backendHost, backendPort, frontendEnvironment, frontendHost, frontendPort, readStartupTimeoutMs } = require("./dev");
const { attachSanitizedLog, openPrivateLog } = require("./dev-daemon/private-log");
const { createProcessToken, isManagedProcessAlive, isPidAlive, processStartTime } = require("./dev-daemon/process-identity");
const { launcherDir, sourceDir } = require("./state");
const { ensureSource } = require("./source");

const serviceId = "ornnlab-dev-service";
const startupTimeoutMs = readStartupTimeoutMs(process.env.ORNNLAB_STARTUP_TIMEOUT_SECONDS);
const restartDelaysMs = readRestartDelaysMs(process.env.ORNNLAB_DEV_RESTART_DELAYS_MS);

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
    lock: path.join(root, "start.lock"),
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
  if (subcommand === "_restart-detached") return restartDetached();
  if (subcommand === "_daemon") return runDaemon();
  return null;
}

async function startDaemon() {
  ensureSource();
  ensureServiceHome();
  const releaseLock = await acquireStartLock();
  try {
    const state = readState();
    if (isManagedProcessAlive(state.daemonPid, state.daemonToken)) {
      const current = await currentStatus();
      console.log(`OrnnLab dev service is ${current.status}.`);
      console.log(`Frontend: ${current.frontendUrl}`);
      console.log(`Backend API: ${current.backendUrl}`);
      return;
    }
    const daemonLog = openPrivateLog(devServicePaths().logs.daemon, ensureServiceHome);
    const daemonToken = createProcessToken();
    const child = spawn(process.execPath, [path.join(__dirname, "..", "bin", "ornnlab.js"), "dev", "_daemon", "--token", daemonToken], {
      cwd: path.join(__dirname, ".."),
      detached: true,
      env: { ...process.env, ORNNLAB_DEV_DAEMON_CHILD: "1", ORNNLAB_DEV_DAEMON_TOKEN: daemonToken },
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
  } finally {
    releaseLock();
  }
}

async function stopDaemon({ quiet = false } = {}) {
  const state = readState();
  if (!isManagedProcessAlive(state.daemonPid, state.daemonToken)) {
    await stopRecordedChildren(state);
    await assertNoUntrustedRecordedServices(state);
    writeState({ status: "stopped", stoppedAt: new Date().toISOString(), desiredState: "stopped" });
    if (!quiet) console.log("OrnnLab dev service is stopped.");
    return;
  }
  await stopManagedPid(state.daemonPid, state.daemonToken);
  await stopRecordedChildren(readState());
  await waitUntilStopped();
  if (!quiet) console.log("OrnnLab dev service is stopped.");
}

function restartDetached() {
  ensureServiceHome();
  const daemonLog = openPrivateLog(devServicePaths().logs.daemon, ensureServiceHome);
  const child = spawn(process.execPath, [path.join(__dirname, "..", "bin", "ornnlab.js"), "dev", "restart"], {
    cwd: path.join(__dirname, ".."),
    detached: true,
    env: { ...process.env, ORNNLAB_DEV_RESTART_HELPER: "1" },
    stdio: ["ignore", daemonLog, daemonLog],
  });
  child.unref();
  fs.closeSync(daemonLog);
  console.log("OrnnLab dev service restart requested.");
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
  const startedAt = new Date().toISOString();
  const daemonToken = process.env.ORNNLAB_DEV_DAEMON_TOKEN || createProcessToken();
  const runtime = { backend: null, frontend: null, restartCount: 0, stopping: false, restartAttempts: { backend: 0, frontend: 0 } };
  writeState(baseState("starting", { backendUrl, frontendUrl, daemonPid: process.pid, daemonToken, daemonStartTime: processStartTime(process.pid), desiredState: "running", restartCount: 0, startedAt }));
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
    markChildRecovered(runtime, "backend");
    markChildRecovered(runtime, "frontend");
    writeRuntimeState(runtime, "running", { backendUrl, frontendUrl });
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
  const token = createProcessToken();
  const child = spawn(process.execPath, [path.join(__dirname, "dev-child-wrapper.js"), "--token", token, "--role", name, "--", command, ...args], {
    cwd,
    detached: process.platform !== "win32",
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.ornnlabStartTime = processStartTime(child.pid);
  child.ornnlabToken = token;
  runtime[name] = child;
  writeState({ ...readState(), [`${name}Pid`]: child.pid, [`${name}Token`]: token, [`${name}StartTime`]: child.ornnlabStartTime });
  attachSanitizedLog(child, logPath, ensureServiceHome);
  try {
    await waitForHealth(healthUrl, name, child);
    child.once("exit", (code, signal) => void onChildExit(name, code, signal, runtime));
    return child;
  } catch (error) {
    await stopChild(child);
    throw error;
  }
}

async function onChildExit(name, code, signal, runtime) {
  if (runtime.stopping) return;
  logEvent("dev_service.child_exited", { child: name, code, signal }, "warn");
  const backendUrl = `http://${backendHost}:${backendPort}`;
  const frontendUrl = `http://${frontendHost}:${frontendPort}`;
  const healthUrl = name === "backend" ? backendUrl : `${frontendUrl}/api/webui/v1/system/health`;
  while (!runtime.stopping && runtime.restartAttempts[name] < restartDelaysMs.length) {
    const attempt = runtime.restartAttempts[name] + 1;
    runtime.restartAttempts[name] = attempt;
    runtime.restartCount += 1;
    const delay = restartDelaysMs[attempt - 1];
    writeState({ ...readState(), status: "degraded", lastError: `${name} exited`, lastRestartAt: new Date().toISOString(), restartCount: runtime.restartCount });
    logEvent("dev_service.restart_scheduled", { child: name, attempt, delayMs: delay }, "warn");
    await sleep(delay);
    if (runtime.stopping) return;
    try {
      await startManagedChild(name, healthUrl, runtime);
      markChildRecovered(runtime, name);
      if (runtimeHasHealthyChildren(runtime)) writeRuntimeState(runtime, "running", { backendUrl, frontendUrl });
      else writeRuntimeState(runtime, "degraded", { backendUrl, frontendUrl, lastError: `${name} recovered; waiting for other service` });
      return;
    } catch (error) {
      writeState({ ...readState(), status: "degraded", lastError: error.message });
      logEvent("dev_service.restart_failed", { child: name, attempt, reason: error.message }, "warn");
    }
  }
  writeState({ ...readState(), status: "error", lastError: `${name} exceeded restart limit` });
  logEvent("dev_service.restart_gave_up", { child: name, attempts: runtime.restartAttempts[name] }, "error");
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
      env: { ...process.env, ORNNLAB_RESTART_COMMAND: `${process.execPath} ${path.join(__dirname, "..", "bin", "ornnlab.js")} dev _restart-detached` },
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

function markChildRecovered(runtime, name) {
  runtime.restartAttempts[name] = 0;
}

function runtimeHasHealthyChildren(runtime) {
  return isManagedProcessAlive(runtime.backend?.pid, runtime.backend?.ornnlabToken)
    && isManagedProcessAlive(runtime.frontend?.pid, runtime.frontend?.ornnlabToken);
}

function writeRuntimeState(runtime, status, values = {}) {
  const current = readState();
  writeState({
    ...baseState(status, { daemonPid: process.pid }),
    ...values,
    daemonToken: current.daemonToken,
    daemonStartTime: processStartTime(process.pid),
    startedAt: current.startedAt || new Date().toISOString(),
    backendPid: runtime.backend?.pid,
    backendToken: runtime.backend?.ornnlabToken,
    backendStartTime: runtime.backend?.ornnlabStartTime,
    frontendPid: runtime.frontend?.pid,
    frontendToken: runtime.frontend?.ornnlabToken,
    frontendStartTime: runtime.frontend?.ornnlabStartTime,
    lastHealthCheckAt: new Date().toISOString(),
    restartCount: runtime.restartCount,
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
  fs.mkdirSync(path.dirname(paths.state), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(paths.state), 0o700);
  fs.mkdirSync(path.dirname(paths.logs.daemon), { recursive: true, mode: 0o700 });
  fs.chmodSync(path.dirname(paths.logs.daemon), 0o700);
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
  fs.chmodSync(devServicePaths().state, 0o600);
}

function logEvent(event, fields = {}, level = "info") {
  ensureServiceHome();
  fs.appendFileSync(devServicePaths().logs.daemon, `${JSON.stringify({ event, level, serviceId, time: new Date().toISOString(), ...fields })}\n`, { mode: 0o600 });
  fs.chmodSync(devServicePaths().logs.daemon, 0o600);
}

async function currentStatus() {
  const state = readState();
  const daemonAlive = isManagedProcessAlive(state.daemonPid, state.daemonToken);
  const backendAlive = isManagedProcessAlive(state.backendPid, state.backendToken);
  const frontendAlive = isManagedProcessAlive(state.frontendPid, state.frontendToken);
  const backendHealthy = backendAlive && await isHealthyEndpoint(`${state.backendUrl || `http://${backendHost}:${backendPort}`}/api/webui/v1/system/health`);
  const frontendHealthy = frontendAlive && await isHealthyEndpoint(state.frontendUrl || `http://${frontendHost}:${frontendPort}`);
  let status = daemonAlive ? state.status : state.status === "error" ? "error" : "stopped";
  if (daemonAlive && state.status === "running" && (!backendHealthy || !frontendHealthy)) status = "degraded";
  return {
    ...state,
    status,
    daemonAlive,
    backendAlive,
    frontendAlive,
    backendHealthy,
    frontendHealthy,
  };
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
    const status = await currentStatus();
    if (!status.daemonAlive && !status.backendAlive && !status.frontendAlive) return;
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
  await Promise.all([
    stopRecordedPid(state.frontendPid, state.frontendToken),
    stopRecordedPid(state.backendPid, state.backendToken),
  ]);
}

async function assertNoUntrustedRecordedServices(state) {
  const checks = [
    ["backend", state.backendPid, state.backendToken, state.backendUrl],
    ["frontend", state.frontendPid, state.frontendToken, state.frontendUrl],
  ];
  for (const [name, pid, token, url] of checks) {
    if (!pid || isManagedProcessAlive(pid, token)) continue;
    if (await isServiceReachable(url)) {
      writeState({ ...state, status: "error", lastError: `${name} has an untrusted managed process still serving ${url}` });
      throw new Error(`${name} has an untrusted managed process still serving ${url}`);
    }
  }
}

async function stopChild(child) {
  if (!child || child.exitCode !== null) return;
  await stopManagedPid(child.pid, child.ornnlabToken);
}

async function signalPid(pid, signal) {
  if (!isPidAlive(pid)) return;
  if (process.platform === "win32") spawnSync("taskkill", ["/pid", String(pid), "/t", "/f"], { stdio: "ignore" });
  else {
    try {
      process.kill(-pid, signal);
    } catch {
      process.kill(pid, signal);
    }
  }
}

async function stopManagedPid(pid, token) {
  if (!isManagedProcessAlive(pid, token)) return;
  await signalPid(pid, "SIGTERM");
  if (await waitForManagedPidExit(pid, token, 3000)) return;
  await signalPid(pid, "SIGKILL");
  if (!(await waitForManagedPidExit(pid, token, 3000))) throw new Error(`Managed process ${pid} did not stop.`);
}

async function isHealthyEndpoint(url) {
  if (!url) return false;
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(500) })).ok;
  } catch {
    return false;
  }
}

async function isServiceReachable(url) {
  if (!url) return false;
  try {
    await fetch(url, { signal: AbortSignal.timeout(500) });
    return true;
  } catch {
    return false;
  }
}

async function stopRecordedPid(pid, token) {
  await stopManagedPid(pid, token);
}

async function waitForManagedPidExit(pid, token, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (!isManagedProcessAlive(pid, token)) return true;
    await sleep(100);
  }
  return !isManagedProcessAlive(pid, token);
}

async function acquireStartLock() {
  const paths = devServicePaths();
  const deadline = Date.now() + startupTimeoutMs;
  while (Date.now() < deadline) {
    try {
      const fd = fs.openSync(paths.lock, "wx", 0o600);
      fs.writeFileSync(fd, `${process.pid}\n`);
      return () => {
        fs.closeSync(fd);
        try { fs.unlinkSync(paths.lock); } catch { /* already removed */ }
      };
    } catch (error) {
      if (error.code !== "EEXIST") throw error;
      const state = readState();
      if (isManagedProcessAlive(state.daemonPid, state.daemonToken)) {
        await waitForDaemonReady();
        continue;
      }
      try {
        if (Date.now() - fs.statSync(paths.lock).mtimeMs > startupTimeoutMs) fs.unlinkSync(paths.lock);
      } catch {
        // lock disappeared between checks
      }
      await sleep(100);
    }
  }
  throw new Error("OrnnLab dev service start lock timed out.");
}

function readRestartDelaysMs(value) {
  if (!value) return [1000, 2000, 5000, 10000, 30000];
  const delays = value.split(",").map((part) => Number(part.trim())).filter((part) => Number.isFinite(part) && part >= 0);
  if (!delays.length) throw new Error("ORNNLAB_DEV_RESTART_DELAYS_MS must contain at least one non-negative number.");
  return delays;
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

module.exports = {
  devServicePaths,
  commandFor,
  handleDevCommand,
  readState,
  startDaemon,
  stopDaemon,
};
