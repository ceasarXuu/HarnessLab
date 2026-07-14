const assert = require("node:assert/strict");
const test = require("node:test");

const { backendInvocation, frontendEnvironment, readStartupTimeoutMs } = require("../../lib/dev");

test("launcher defaults the managed frontend to API mode", () => {
  const original = process.env.VITE_ORNNLAB_DATA_MODE;
  delete process.env.VITE_ORNNLAB_DATA_MODE;
  try {
    const environment = frontendEnvironment("http://127.0.0.1:19000");
    assert.equal(environment.VITE_ORNNLAB_DATA_MODE, "api");
    assert.equal(environment.ORNNLAB_API_TARGET, "http://127.0.0.1:19000");
  } finally {
    if (original === undefined) delete process.env.VITE_ORNNLAB_DATA_MODE;
    else process.env.VITE_ORNNLAB_DATA_MODE = original;
  }
});

test("launcher rejects invalid data modes and startup timeouts", () => {
  const original = process.env.VITE_ORNNLAB_DATA_MODE;
  process.env.VITE_ORNNLAB_DATA_MODE = "preview";
  try {
    assert.throws(() => frontendEnvironment(), /VITE_ORNNLAB_DATA_MODE/);
  } finally {
    if (original === undefined) delete process.env.VITE_ORNNLAB_DATA_MODE;
    else process.env.VITE_ORNNLAB_DATA_MODE = original;
  }
  assert.equal(readStartupTimeoutMs(undefined), 300000);
  assert.equal(readStartupTimeoutMs("12"), 12000);
  assert.throws(() => readStartupTimeoutMs("0"), /ORNNLAB_STARTUP_TIMEOUT_SECONDS/);
});

test("launcher prefers the local virtualenv backend entrypoint when available", () => {
  const backend = backendInvocation(["web"]);

  if (backend.command.endsWith(".venv/bin/ornnlab") || backend.command.endsWith(".venv\\Scripts\\ornnlab.exe")) {
    assert.deepEqual(backend.args, ["web"]);
    return;
  }
  assert.equal(backend.command, "uv");
  assert.deepEqual(backend.args, ["run", "ornnlab", "web"]);
});

test("launcher bypasses local dev endpoints from system proxy settings", () => {
  const { ensureLocalhostNoProxy } = require("../../lib/dev");
  const env = { NO_PROXY: "example.com,localhost", no_proxy: "" };

  ensureLocalhostNoProxy(env);

  assert.equal(env.NO_PROXY, "example.com,localhost,127.0.0.1,::1");
  assert.equal(env.no_proxy, "127.0.0.1,localhost,::1");
});
