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

    assert.match(backend.env.ORNNLAB_RESTART_COMMAND, /ornnlab\.js dev restart$/);
  } finally {
    if (original === undefined) delete process.env.ORNNLAB_DEV_BACKEND_COMMAND;
    else process.env.ORNNLAB_DEV_BACKEND_COMMAND = original;
  }
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
