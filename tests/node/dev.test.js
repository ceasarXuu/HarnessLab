const assert = require("node:assert/strict");
const test = require("node:test");

const { frontendEnvironment, readStartupTimeoutMs } = require("../../lib/dev");

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
  assert.equal(readStartupTimeoutMs(undefined), 30000);
  assert.equal(readStartupTimeoutMs("12"), 12000);
  assert.throws(() => readStartupTimeoutMs("0"), /ORNNLAB_STARTUP_TIMEOUT_SECONDS/);
});
