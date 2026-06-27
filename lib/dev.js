const { spawnAttached, run } = require("./common");
const { sourceDir } = require("./state");
const { ensureSource } = require("./source");
const { ensureReady } = require("./bootstrap");

const backendHost = process.env.ORNNLAB_BACKEND_HOST || "127.0.0.1";
const backendPort = process.env.ORNNLAB_BACKEND_PORT || "8765";

function runBackend(args) {
  ensureSource();
  run("uv", ["run", "ornnlab", "web", "--host", backendHost, "--port", backendPort, ...args], {
    cwd: sourceDir,
  });
}

function runDoctor(args) {
  ensureSource();
  run("uv", ["run", "ornnlab", "doctor", ...args], { cwd: sourceDir });
}

function runFrontend(args) {
  void args;
  throw new Error(
    "The legacy Vue frontend has been removed. Rebuild the v1.0.5 UI with the official Harbor viewer-aligned architecture before using `ornnlab ui`.",
  );
}

function printLaunchUrls() {
  console.log("");
  console.log("OrnnLab backend is starting.");
  console.log(`Backend API: http://${backendHost}:${backendPort}/`);
  console.log("Frontend: pending v1.0.5 rebuild against the official Harbor viewer architecture.");
  console.log("Press Ctrl+C to stop the server.");
  console.log("");
}

async function runDev({ setupIfMissing = false } = {}) {
  if (setupIfMissing) {
    await ensureReady();
  } else {
    ensureSource();
  }
  printLaunchUrls();
  const backend = spawnAttached(
    "uv",
    ["run", "ornnlab", "web", "--host", backendHost, "--port", backendPort],
    { cwd: sourceDir },
  );
  const shutdown = () => {
    backend.kill("SIGTERM");
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

module.exports = {
  backendHost,
  backendPort,
  runBackend,
  runDoctor,
  runFrontend,
  printLaunchUrls,
  runDev,
};
