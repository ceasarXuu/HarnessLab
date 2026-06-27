const fs = require("node:fs");
const path = require("node:path");
const { run, phase } = require("./common");
const { launcherDir, sourceDir, saveState } = require("./state");

const repoUrl = process.env.ORNNLAB_REPO || "https://github.com/ceasarXuu/HarnessLab.git";

function ensureSource() {
  if (!fs.existsSync(sourceDir)) {
    throw new Error("Source checkout not found. Run: ornnlab install");
  }
  const gitDir = path.join(sourceDir, ".git");
  if (!fs.existsSync(gitDir)) {
    throw new Error(`Source path exists but is not a git checkout: ${sourceDir}`);
  }
}

function sourceReady() {
  return fs.existsSync(path.join(sourceDir, ".git"));
}

function backendReady() {
  return fs.existsSync(path.join(sourceDir, ".venv"));
}

function frontendReady() {
  const packageJson = path.join(sourceDir, "frontend", "package.json");
  if (!fs.existsSync(packageJson)) {
    return true;
  }
  return fs.existsSync(path.join(sourceDir, "frontend", "node_modules"));
}

function ensureProjectSource() {
  phase("Preparing OrnnLab source checkout");
  fs.mkdirSync(launcherDir, { recursive: true });
  if (!fs.existsSync(sourceDir)) {
    run("git", ["clone", repoUrl, sourceDir]);
  } else {
    ensureSource();
    run("git", ["pull", "--ff-only"], { cwd: sourceDir });
  }
  saveState({ source: { status: "ready", path: sourceDir } });
}

function syncBackendDependencies() {
  phase("Syncing Python backend dependencies");
  run("uv", ["sync", "--group", "dev"], { cwd: sourceDir });
  phase("Verifying Python backend dependencies");
  run("uv", ["run", "python", "-c", "import harbor; import ornnlab"], { cwd: sourceDir });
  run("uv", ["run", "ornnlab", "--version"], { cwd: sourceDir, stdio: "ignore" });
  saveState({ backend: { status: "ready", verified: true } });
}

function syncFrontendDependencies() {
  phase("Skipping frontend dependency sync");
  console.log(
    "The legacy Vue frontend has been removed. v1.0.5 will rebuild the UI against the official Harbor viewer architecture.",
  );
  saveState({ frontend: { status: "pending-rebuild", verified: false } });
}

module.exports = {
  repoUrl,
  ensureSource,
  sourceReady,
  backendReady,
  frontendReady,
  ensureProjectSource,
  syncBackendDependencies,
  syncFrontendDependencies,
};
