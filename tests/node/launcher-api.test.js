const assert = require("node:assert/strict");
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const repoRoot = path.resolve(__dirname, "../..");

test("ornnlab dev starts an API-mode frontend proxy", { timeout: 60000 }, async (context) => {
  const [backendPort, frontendPort] = await Promise.all([freePort(), freePort()]);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ornnlab-launcher-api-"));
  const child = spawn(process.execPath, ["bin/ornnlab.js", "dev"], {
    cwd: repoRoot,
    env: {
      ...process.env,
      ORNNLAB_BACKEND_PORT: String(backendPort),
      ORNNLAB_FRONTEND_PORT: String(frontendPort),
      ORNNLAB_HOME: path.join(tempRoot, "data"),
      ORNNLAB_LAUNCHER_HOME: path.join(tempRoot, "launcher"),
      ORNNLAB_SOURCE: repoRoot,
      ORNNLAB_STARTUP_TIMEOUT_SECONDS: "30",
    },
  });
  let output = "";
  child.stdout.on("data", (chunk) => { output += chunk; });
  child.stderr.on("data", (chunk) => { output += chunk; });

  const url = `http://127.0.0.1:${frontendPort}/api/webui/v1/system/health`;
  context.after(async () => {
    await stop(child);
    await waitForUnavailable(url);
  });
  await waitForOk(url, child, () => output);
  await waitForText(/Frontend: http:\/\/127\.0\.0\.1:/, child, () => output);
  const response = await fetch(url);
  const payload = await response.json();

  assert.equal(response.ok, true);
  assert.ok(Array.isArray(payload.data.items));
});

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

async function waitForOk(url, child, getOutput) {
  const deadline = Date.now() + 30000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`Launcher exited before the API proxy became ready:\n${getOutput()}`);
    }
    try {
      if ((await fetch(url)).ok) return;
    } catch {
      // The local processes may still be binding their ports.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Launcher API proxy did not become ready:\n${getOutput()}`);
}

async function waitForText(pattern, child, getOutput) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    if (pattern.test(getOutput())) return;
    if (child.exitCode !== null) break;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Launcher did not print its ready summary:\n${getOutput()}`);
}

async function waitForUnavailable(url) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    try {
      await fetch(url);
    } catch {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(`Launcher left the frontend proxy running at ${url}.`);
}

function stop(child) {
  if (child.exitCode !== null) return Promise.resolve();
  child.kill("SIGTERM");
  return new Promise((resolve) => child.once("exit", resolve));
}
