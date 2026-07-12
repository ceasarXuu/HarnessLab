const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const repoRoot = path.resolve(__dirname, "../..");
const fakeService = path.join(repoRoot, "tests/node/fixtures/fake-dev-service.js");

test("dev service state lives outside the launcher state file", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-state-"));
  const original = process.env.ORNNLAB_DEV_SERVICE_HOME;
  process.env.ORNNLAB_DEV_SERVICE_HOME = path.join(tempRoot, "dev-service");
  const { devServicePaths } = require("../../lib/dev-daemon");
  try {
    const paths = devServicePaths();
    assert.equal(paths.root, path.join(tempRoot, "dev-service"));
    assert.equal(paths.state, path.join(tempRoot, "dev-service", "state.json"));
    assert.equal(paths.logs.daemon, path.join(tempRoot, "dev-service", "logs", "daemon.log"));
  } finally {
    if (original === undefined) delete process.env.ORNNLAB_DEV_SERVICE_HOME;
    else process.env.ORNNLAB_DEV_SERVICE_HOME = original;
  }
});

test("ornnlab dev start launches a daemon that can report status and stop", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-daemon-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);
  assert.match(start.stdout, /OrnnLab dev service is running/);

  const status = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "status", "--json"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(status.status, 0, status.stderr || status.stdout);
  const payload = JSON.parse(status.stdout);
  assert.equal(payload.status, "running");
  assert.equal(payload.backendUrl, `http://127.0.0.1:${backendPort}`);
  assert.equal(payload.frontendUrl, `http://127.0.0.1:${frontendPort}`);
  assert.equal((await fetch(`${payload.frontendUrl}/api/webui/v1/system/health`)).ok, true);

  const stop = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(stop.status, 0, stop.stderr || stop.stdout);
  await waitForUnavailable(`http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`);
});

test("default backend child receives the daemon restart command", () => {
  const original = process.env.ORNNLAB_DEV_BACKEND_COMMAND;
  delete process.env.ORNNLAB_DEV_BACKEND_COMMAND;
  try {
    const { commandFor } = require("../../lib/dev-daemon");
    const backend = commandFor("backend");

    assert.match(backend.env.ORNNLAB_RESTART_COMMAND, /ornnlab\.js dev _restart-detached$/);
  } finally {
    if (original === undefined) delete process.env.ORNNLAB_DEV_BACKEND_COMMAND;
    else process.env.ORNNLAB_DEV_BACKEND_COMMAND = original;
  }
});

test("dev stop refuses to kill pids that are not proven daemon children", async () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-stale-pid-"));
  const env = daemonEnv(tempRoot, await freePort(), await freePort());
  const sleeper = spawn("sleep", ["30"], { stdio: "ignore" });
  const statePath = path.join(tempRoot, "dev-service", "state.json");
  fs.mkdirSync(path.dirname(statePath), { recursive: true });
  fs.writeFileSync(statePath, JSON.stringify({ status: "running", backendPid: sleeper.pid }, null, 2));

  const stop = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });

  assert.equal(stop.status, 0, stop.stderr || stop.stdout);
  assert.equal(isProcessAlive(sleeper.pid), true);
  sleeper.kill("SIGTERM");
});

test("dev status does not trust a live pid without recorded process identity", async () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-fake-daemon-"));
  const env = daemonEnv(tempRoot, await freePort(), await freePort());
  const sleeper = spawn("sleep", ["30"], { stdio: "ignore" });
  const statePath = path.join(tempRoot, "dev-service", "state.json");
  fs.mkdirSync(path.dirname(statePath), { recursive: true });
  fs.writeFileSync(statePath, JSON.stringify({ status: "running", daemonPid: sleeper.pid }, null, 2));

  const status = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "status", "--json"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  const payload = JSON.parse(status.stdout);

  assert.equal(payload.status, "stopped");
  assert.equal(payload.daemonAlive, false);
  sleeper.kill("SIGTERM");
});

test("concurrent dev start keeps a single daemon instance", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-singleton-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const [first, second] = await Promise.all([
    runLauncher(["dev", "start"], env),
    runLauncher(["dev", "start"], env),
  ]);

  assert.equal(first.status, 0, first.stderr || first.stdout);
  assert.equal(second.status, 0, second.stderr || second.stdout);
  const logText = fs.readFileSync(path.join(tempRoot, "dev-service", "logs", "daemon.log"), "utf8");
  assert.equal((logText.match(/dev_service\.start_requested/g) || []).length, 1);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("dev daemon records startup failure and exits instead of idling forever", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-error-"));
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_FRONTEND_COMMAND: `${process.execPath} -e "process.exit(42)"`,
    ORNNLAB_STARTUP_TIMEOUT_SECONDS: "3",
  };
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.notEqual(start.status, 0);

  const state = JSON.parse(fs.readFileSync(path.join(tempRoot, "dev-service", "state.json"), "utf8"));
  assert.equal(state.status, "error");
  assert.equal(isProcessAlive(state.daemonPid), false);
  const logText = fs.readFileSync(path.join(tempRoot, "dev-service", "logs", "daemon.log"), "utf8");
  assert.match(logText, /dev_service\.restart_gave_up/);
});

test("dev stop during startup terminates children that are not healthy yet", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-start-stop-"));
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_FRONTEND_COMMAND: `${process.execPath} ${fakeService} --role frontend --host 127.0.0.1 --port ${frontendPort} --delay-health 10000`,
  };
  const starting = runLauncher(["dev", "start"], env);
  await waitForState(path.join(tempRoot, "dev-service", "state.json"), "starting");

  const stop = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });

  assert.equal(stop.status, 0, stop.stderr || stop.stdout);
  await waitForUnavailable(`http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`);
  await starting;
});

test("dev daemon restarts a crashed frontend child", { timeout: 60000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-restart-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);

  const statePath = path.join(tempRoot, "dev-service", "state.json");
  const before = JSON.parse(fs.readFileSync(statePath, "utf8"));
  process.kill(before.frontendPid, "SIGTERM");
  await waitForRestart(statePath, "frontendPid", before.frontendPid);
  await waitForOk(`http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`);
  const logText = fs.readFileSync(path.join(tempRoot, "dev-service", "logs", "daemon.log"), "utf8");
  assert.match(logText, /dev_service\.child_exited/);
  assert.match(logText, /dev_service\.restart_scheduled/);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("dev daemon restarts a crashed backend child", { timeout: 60000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-backend-restart-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);

  const statePath = path.join(tempRoot, "dev-service", "state.json");
  const before = JSON.parse(fs.readFileSync(statePath, "utf8"));
  process.kill(before.backendPid, "SIGTERM");
  await waitForRestart(statePath, "backendPid", before.backendPid);
  await waitForOk(`http://127.0.0.1:${backendPort}/api/webui/v1/system/health`);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("detached restart helper replaces the daemon and restores health", { timeout: 60000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-detached-restart-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);

  const statePath = path.join(tempRoot, "dev-service", "state.json");
  const before = JSON.parse(fs.readFileSync(statePath, "utf8"));
  const restart = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "_restart-detached"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(restart.status, 0, restart.stderr || restart.stdout);
  await waitForRestart(statePath, "daemonPid", before.daemonPid);
  await waitForOk(`http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("dev daemon keeps retrying after a crash restart fails", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-retry-failure-"));
  const marker = path.join(tempRoot, "frontend-first-start");
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_FRONTEND_COMMAND: `${process.execPath} ${fakeService} --role frontend --host 127.0.0.1 --port ${frontendPort} --fail-after-first ${marker}`,
    ORNNLAB_DEV_RESTART_DELAYS_MS: "25,25",
  };
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);

  const statePath = path.join(tempRoot, "dev-service", "state.json");
  const before = JSON.parse(fs.readFileSync(statePath, "utf8"));
  process.kill(before.frontendPid, "SIGTERM");
  await waitForState(statePath, "error");
  const logText = fs.readFileSync(path.join(tempRoot, "dev-service", "logs", "daemon.log"), "utf8");
  assert.match(logText, /"attempt":1/);
  assert.match(logText, /"attempt":2/);
  assert.match(logText, /dev_service\.restart_gave_up/);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("dev daemon redacts sensitive child output and restricts log permissions", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-log-security-"));
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_BACKEND_COMMAND: `${process.execPath} ${fakeService} --role backend --host 127.0.0.1 --port ${backendPort} --print-secret super-secret`,
  };
  const start = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], {
    cwd: repoRoot,
    encoding: "utf-8",
    env,
  });
  assert.equal(start.status, 0, start.stderr || start.stdout);
  const backendLog = path.join(tempRoot, "dev-service", "logs", "backend.log");
  const logText = fs.readFileSync(backendLog, "utf8");
  const mode = fs.statSync(backendLog).mode & 0o777;

  assert.doesNotMatch(logText, /super-secret/);
  assert.match(logText, /ANTHROPIC_API_KEY=\[REDACTED\]/);
  assert.equal(mode, 0o600);

  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

function daemonEnv(tempRoot, backendPort, frontendPort) {
  return {
    ...process.env,
    ORNNLAB_BACKEND_PORT: String(backendPort),
    ORNNLAB_DEV_BACKEND_COMMAND: `${process.execPath} ${fakeService} --role backend --host 127.0.0.1 --port ${backendPort}`,
    ORNNLAB_DEV_FRONTEND_COMMAND: `${process.execPath} ${fakeService} --role frontend --host 127.0.0.1 --port ${frontendPort}`,
    ORNNLAB_DEV_SERVICE_HOME: path.join(tempRoot, "dev-service"),
    ORNNLAB_FRONTEND_PORT: String(frontendPort),
    ORNNLAB_LAUNCHER_HOME: path.join(tempRoot, "launcher"),
    ORNNLAB_SOURCE: repoRoot,
    ORNNLAB_STARTUP_TIMEOUT_SECONDS: "20",
  };
}

function runLauncher(args, env) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, ["bin/ornnlab.js", ...args], {
      cwd: repoRoot,
      encoding: "utf-8",
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => { stdout += chunk; });
    child.stderr.on("data", (chunk) => { stderr += chunk; });
    child.on("exit", (status) => resolve({ status, stdout, stderr }));
  });
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = http.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      server.close((error) => error ? reject(error) : resolve(port));
    });
  });
}

async function waitForOk(url) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {
      // waiting for local service
    }
    await sleep(250);
  }
  throw new Error(`URL did not become ready: ${url}`);
}

async function waitForUnavailable(url) {
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    try {
      await fetch(url);
    } catch {
      return;
    }
    await sleep(100);
  }
  throw new Error(`URL remained available: ${url}`);
}

async function waitForRestart(statePath, field, previousPid) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const state = JSON.parse(fs.readFileSync(statePath, "utf8"));
    if (state[field] && state[field] !== previousPid && state.status === "running") return;
    await sleep(250);
  }
  throw new Error(`${field} did not restart`);
}

async function waitForState(statePath, expected) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    try {
      const state = JSON.parse(fs.readFileSync(statePath, "utf8"));
      if (state.status === expected) return state;
    } catch {
      // state may not be written yet
    }
    await sleep(100);
  }
  throw new Error(`state did not become ${expected}`);
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function isProcessAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}
