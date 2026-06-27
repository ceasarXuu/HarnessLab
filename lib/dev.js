const path = require("node:path");
const { spawnAttached, run } = require("./common");
const { sourceDir } = require("./state");
const { ensureSource } = require("./source");
const { ensureReady } = require("./bootstrap");

const backendHost = process.env.ORNNLAB_BACKEND_HOST || "127.0.0.1";
const backendPort = process.env.ORNNLAB_BACKEND_PORT || "8765";
const frontendHost = process.env.ORNNLAB_FRONTEND_HOST || "127.0.0.1";
const frontendPort = process.env.ORNNLAB_FRONTEND_PORT || "5173";

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
  ensureSource();
  run("npm", ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort", ...args], {
    cwd: path.join(sourceDir, "frontend"),
  });
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
  printLaunchUrls();
  const backend = spawnAttached(
    "uv",
    ["run", "ornnlab", "web", "--host", backendHost, "--port", backendPort],
    { cwd: sourceDir },
  );
  const frontend = spawnAttached(
    "npm",
    ["run", "dev", "--", "--host", frontendHost, "--port", frontendPort, "--strictPort"],
    { cwd: path.join(sourceDir, "frontend") },
  );
  const shutdown = () => {
    backend.kill("SIGTERM");
    frontend.kill("SIGTERM");
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

module.exports = {
  backendHost,
  backendPort,
  frontendHost,
  frontendPort,
  runBackend,
  runDoctor,
  runFrontend,
  printLaunchUrls,
  runDev,
};
