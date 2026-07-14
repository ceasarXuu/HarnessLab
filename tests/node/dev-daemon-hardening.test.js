const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const repoRoot = path.resolve(__dirname, "../..");
const fakeService = path.join(repoRoot, "tests/node/fixtures/fake-dev-service.js");

test("dev stop kills managed children that ignore SIGTERM", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-ignore-term-"));
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_FRONTEND_COMMAND: `${process.execPath} ${fakeService} --role frontend --host 127.0.0.1 --port ${frontendPort} --ignore-sigterm`,
  };
  assert.equal(spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], { cwd: repoRoot, env }).status, 0);

  const state = JSON.parse(fs.readFileSync(statePath(tempRoot), "utf8"));
  assert.equal(isProcessAlive(state.frontendPid), true);
  const stop = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, encoding: "utf8", env });

  assert.equal(stop.status, 0, stop.stderr || stop.stdout);
  await waitForUnavailable(`http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`);
  assert.equal(isProcessAlive(state.frontendPid), false);
});

test("dev child wrapper terminates descendant services", { timeout: 20000 }, async () => {
  if (process.platform === "win32") return;
  const port = await freePort();
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-wrapper-tree-"));
  const serverScript = path.join(tempRoot, "server.js");
  const parentScript = path.join(tempRoot, "parent.js");
  fs.writeFileSync(serverScript, `
const http = require("node:http");
const port = Number(process.argv[2]);
http.createServer((request, response) => {
  response.writeHead(200, { "content-type": "application/json" });
  response.end(JSON.stringify({ ok: true }));
}).listen(port, "127.0.0.1");
setInterval(() => {}, 30000);
`);
  fs.writeFileSync(parentScript, `
const { spawn } = require("node:child_process");
const child = spawn(process.execPath, [${JSON.stringify(serverScript)}, process.argv[2]], { stdio: "ignore" });
child.unref();
setInterval(() => {}, 30000);
`);
  const wrapper = spawn(process.execPath, [
    path.join(repoRoot, "lib/dev-child-wrapper.js"),
    "--token",
    "wrapper-tree-test-token",
    "--",
    process.execPath,
    parentScript,
    String(port),
  ], { cwd: repoRoot, stdio: "ignore" });
  await waitForOk(`http://127.0.0.1:${port}/`);

  wrapper.kill("SIGTERM");

  await waitForUnavailable(`http://127.0.0.1:${port}/`);
  assert.equal(await waitForExit(wrapper), true);
});

test("dev status degrades when daemon is alive but a child is missing", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-degraded-status-"));
  const env = { ...daemonEnv(tempRoot, backendPort, frontendPort), ORNNLAB_DEV_RESTART_DELAYS_MS: "5000" };
  assert.equal(spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], { cwd: repoRoot, env }).status, 0);

  const before = JSON.parse(fs.readFileSync(statePath(tempRoot), "utf8"));
  process.kill(before.frontendPid, "SIGKILL");
  await waitForState(statePath(tempRoot), "degraded");

  const status = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "status", "--json"], {
    cwd: repoRoot,
    encoding: "utf8",
    env,
  });
  const payload = JSON.parse(status.stdout);

  assert.equal(payload.daemonAlive, true);
  assert.equal(payload.frontendAlive, false);
  assert.equal(payload.status, "degraded");
  spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], { cwd: repoRoot, env });
});

test("dev status requires managed child health, not only pid and token", async () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-fake-health-"));
  const env = daemonEnv(tempRoot, await freePort(), await freePort());
  const token = "shared-health-token";
  const sleeper = spawn(process.execPath, ["-e", `setTimeout(()=>{},30000)`, token], { stdio: "ignore" });
  fs.mkdirSync(path.dirname(statePath(tempRoot)), { recursive: true });
  fs.writeFileSync(statePath(tempRoot), JSON.stringify({
    status: "running",
    daemonPid: sleeper.pid,
    daemonToken: token,
    backendPid: sleeper.pid,
    backendToken: token,
    frontendPid: sleeper.pid,
    frontendToken: token,
    backendUrl: `http://127.0.0.1:${env.ORNNLAB_BACKEND_PORT}`,
    frontendUrl: `http://127.0.0.1:${env.ORNNLAB_FRONTEND_PORT}`,
  }));

  const status = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "status", "--json"], {
    cwd: repoRoot,
    encoding: "utf8",
    env,
  });
  const payload = JSON.parse(status.stdout);

  assert.equal(payload.daemonAlive, true);
  assert.equal(payload.backendAlive, true);
  assert.equal(payload.frontendAlive, true);
  assert.equal(payload.status, "degraded");
  sleeper.kill("SIGTERM");
});

test("dev status rejects same-pid records with a wrong process token", async () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-wrong-token-"));
  const env = daemonEnv(tempRoot, await freePort(), await freePort());
  const sleeper = spawn("sleep", ["30"], { stdio: "ignore" });
  fs.mkdirSync(path.dirname(statePath(tempRoot)), { recursive: true });
  fs.writeFileSync(statePath(tempRoot), JSON.stringify({
    status: "running",
    daemonPid: sleeper.pid,
    daemonToken: "wrong-token",
  }));

  const status = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "status", "--json"], {
    cwd: repoRoot,
    encoding: "utf8",
    env,
  });
  const payload = JSON.parse(status.stdout);

  assert.equal(payload.daemonAlive, false);
  assert.equal(payload.status, "stopped");
  sleeper.kill("SIGTERM");
});

test("dev stop fails closed when recorded untrusted child port is still serving", async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-untrusted-port-"));
  const env = daemonEnv(tempRoot, backendPort, frontendPort);
  const server = spawn(process.execPath, [fakeService, "--role", "backend", "--host", "127.0.0.1", "--port", String(backendPort)], { stdio: "ignore" });
  await waitForOk(`http://127.0.0.1:${backendPort}/api/webui/v1/system/health`);
  fs.mkdirSync(path.dirname(statePath(tempRoot)), { recursive: true });
  fs.writeFileSync(statePath(tempRoot), JSON.stringify({
    status: "running",
    backendPid: server.pid,
    backendToken: "wrong-token",
    backendUrl: `http://127.0.0.1:${backendPort}`,
    frontendUrl: `http://127.0.0.1:${frontendPort}`,
  }));

  const stop = spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "stop"], {
    cwd: repoRoot,
    encoding: "utf8",
    env,
  });

  assert.notEqual(stop.status, 0);
  assert.match(stop.stderr || stop.stdout, /untrusted managed process/i);
  assert.equal((await fetch(`http://127.0.0.1:${backendPort}/api/webui/v1/system/health`)).ok, true);
  server.kill("SIGTERM");
});

test("dev daemon fixes existing log permissions and redacts split secrets", { timeout: 45000 }, async () => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-dev-log-hardening-"));
  const backendLog = path.join(tempRoot, "dev-service", "logs", "backend.log");
  fs.mkdirSync(path.dirname(backendLog), { recursive: true });
  fs.writeFileSync(backendLog, "old log\n", { mode: 0o644 });
  fs.chmodSync(backendLog, 0o644);
  const env = {
    ...daemonEnv(tempRoot, backendPort, frontendPort),
    ORNNLAB_DEV_BACKEND_COMMAND: `${process.execPath} ${fakeService} --role backend --host 127.0.0.1 --port ${backendPort} --split-secret chunk-secret --print-authorization bearer-secret`,
  };

  assert.equal(spawnSync(process.execPath, ["bin/ornnlab.js", "dev", "start"], { cwd: repoRoot, env }).status, 0);
  const logText = fs.readFileSync(backendLog, "utf8");
  const mode = fs.statSync(backendLog).mode & 0o777;

  assert.equal(mode, 0o600);
  assert.doesNotMatch(logText, /chunk-secret|bearer-secret/);
  assert.match(logText, /ANTHROPIC_API_KEY=\[REDACTED\]/);
  assert.match(logText, /Authorization: Bearer \[REDACTED\]/);
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

function statePath(tempRoot) {
  return path.join(tempRoot, "dev-service", "state.json");
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
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {
      // waiting for local service
    }
    await sleep(100);
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

async function waitForState(filePath, expected) {
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    const state = JSON.parse(fs.readFileSync(filePath, "utf8"));
    if (state.status === expected) return state;
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

function waitForExit(child) {
  if (child.exitCode !== null) return Promise.resolve(true);
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve(false), 5000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve(true);
    });
  });
}
