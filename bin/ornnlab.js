#!/usr/bin/env node

const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const readline = require("node:readline");
const { version: packageVersion } = require("../package.json");

const repoUrl = process.env.ORNNLAB_REPO || "https://github.com/ceasarXuu/HarnessLab.git";
const launcherDir = process.env.ORNNLAB_LAUNCHER_HOME || path.join(os.homedir(), ".ornnlab", "launcher");
const sourceDir = process.env.ORNNLAB_SOURCE || path.join(launcherDir, "source");
const statePath = path.join(launcherDir, "bootstrap-state.json");
const backendHost = process.env.ORNNLAB_BACKEND_HOST || "127.0.0.1";
const backendPort = process.env.ORNNLAB_BACKEND_PORT || "8765";
const frontendHost = process.env.ORNNLAB_FRONTEND_HOST || "127.0.0.1";
const frontendPort = process.env.ORNNLAB_FRONTEND_PORT || "5173";

function addPathIfPresent(candidate) {
  if (fs.existsSync(candidate)) {
    const parts = process.env.PATH.split(path.delimiter);
    if (!parts.includes(candidate)) {
      process.env.PATH = `${candidate}${path.delimiter}${process.env.PATH}`;
    }
  }
}

function refreshBootstrapPath() {
  addPathIfPresent(path.join(os.homedir(), ".local", "bin"));
  addPathIfPresent("/opt/homebrew/bin");
  addPathIfPresent("/usr/local/bin");
}

refreshBootstrapPath();

const help = `OrnnLab npm launcher

Usage:
  ornnlab                    Bootstrap if needed, then start the local WebUI
  ornnlab install            Install prerequisites, clone/update source, and install dependencies
  ornnlab setup              Alias for install
  ornnlab dev                Start backend and frontend development servers
  ornnlab web [args...]      Start the FastAPI backend from the managed source checkout
  ornnlab ui [args...]       Start the Vue frontend dev server from the managed source checkout
  ornnlab doctor [args...]   Run OrnnLab doctor from the managed source checkout
  ornnlab path               Print the managed source checkout path
  ornnlab --version          Print launcher version
  ornnlab --help             Show this help

Environment:
  ORNNLAB_LAUNCHER_HOME   Default: ~/.ornnlab/launcher
  ORNNLAB_SOURCE          Default: ~/.ornnlab/launcher/source
  ORNNLAB_REPO            Default: ${repoUrl}
  ORNNLAB_BACKEND_PORT    Default: 8765
  ORNNLAB_FRONTEND_PORT   Default: 5173
  ORNNLAB_INSTALL_DOCKER  1=yes, 0=no, unset=ask when missing
  ORNNLAB_AUTO_INSTALL    0=diagnose only; default installs missing required tools

Bootstrap behavior:
  Required tools are git, uv, Node.js, and npm. The launcher tries to install
  missing required tools on macOS, Linux, and Windows when a supported package
  manager is available. Docker is optional for first launch; if it is missing,
  the launcher asks whether to install it and records the choice.

When the app starts, the terminal prints:
  Frontend: http://${frontendHost}:${frontendPort}/
`;

function phase(message) {
  console.log(`\n==> ${message}`);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: process.env,
    shell: options.shell || false,
    stdio: options.stdio || "inherit",
  });
  if (result.error) {
    throw new Error(`${command} failed to start: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
  return result;
}

function runShell(command, options = {}) {
  return run(command, [], { ...options, shell: true });
}

function spawnAttached(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: options.cwd,
    env: process.env,
    stdio: "inherit",
  });
  child.on("error", (error) => {
    console.error(`${command} failed to start: ${error.message}`);
    process.exitCode = 1;
  });
  return child;
}

function commandExists(command) {
  const probe = spawnSync(command, ["--version"], { stdio: "ignore", shell: process.platform === "win32" });
  return !probe.error && probe.status === 0;
}

function loadState() {
  try {
    return JSON.parse(fs.readFileSync(statePath, "utf8"));
  } catch {
    return {};
  }
}

function saveState(patch) {
  fs.mkdirSync(launcherDir, { recursive: true });
  const state = {
    ...loadState(),
    ...patch,
    platform: process.platform,
    updatedAt: new Date().toISOString(),
  };
  fs.writeFileSync(statePath, `${JSON.stringify(state, null, 2)}\n`);
}

function sudoPrefix() {
  return process.getuid && process.getuid() === 0 ? "" : "sudo ";
}

function linuxInstaller(command) {
  const sudo = sudoPrefix();
  if (commandExists("apt-get")) return `${sudo}apt-get update && ${sudo}apt-get install -y ${command}`;
  if (commandExists("dnf")) return `${sudo}dnf install -y ${command}`;
  if (commandExists("yum")) return `${sudo}yum install -y ${command}`;
  if (commandExists("pacman")) return `${sudo}pacman -Sy --noconfirm ${command}`;
  if (commandExists("zypper")) return `${sudo}zypper install -y ${command}`;
  if (commandExists("apk")) return `${sudo}apk add ${command}`;
  return null;
}

function installHomebrewIfNeeded() {
  if (commandExists("brew")) return;
  phase("Installing Homebrew because no supported macOS package manager was found");
  runShell('/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"');
}

function installRequiredTool(tool) {
  if (process.env.ORNNLAB_AUTO_INSTALL === "0") {
    throw new Error(`Required command not available on PATH: ${tool}`);
  }

  phase(`Installing required tool: ${tool}`);
  if (process.platform === "darwin") {
    installHomebrewIfNeeded();
    const brewName = tool === "npm" || tool === "node" ? "node" : tool;
    run("brew", ["install", brewName]);
    return;
  }

  if (process.platform === "linux") {
    if (tool === "uv") {
      runShell('curl -LsSf https://astral.sh/uv/install.sh | sh');
      refreshBootstrapPath();
      return;
    }
    const packageName = tool === "npm" || tool === "node" ? "nodejs npm" : tool;
    const command = linuxInstaller(packageName);
    if (!command) {
      throw new Error(`No supported Linux package manager found to install ${tool}`);
    }
    runShell(command);
    return;
  }

  if (process.platform === "win32") {
    if (!commandExists("winget")) {
      throw new Error(`winget is required to install ${tool} automatically on Windows`);
    }
    const ids = {
      git: "Git.Git",
      uv: "astral-sh.uv",
      node: "OpenJS.NodeJS.LTS",
      npm: "OpenJS.NodeJS.LTS",
    };
    run("winget", ["install", "--id", ids[tool], "-e", "--source", "winget"]);
    return;
  }

  throw new Error(`Unsupported platform for automatic ${tool} installation: ${process.platform}`);
}

function ensureRequiredTool(tool, probe = tool) {
  if (!commandExists(probe)) {
    installRequiredTool(tool);
  }
  if (!commandExists(probe)) {
    throw new Error(`Required command still unavailable after install attempt: ${probe}`);
  }
}

function installDocker() {
  phase("Installing optional Docker capability");
  if (process.platform === "darwin") {
    installHomebrewIfNeeded();
    run("brew", ["install", "--cask", "docker"]);
    return;
  }
  if (process.platform === "linux") {
    const command = linuxInstaller("docker.io");
    if (!command) throw new Error("No supported Linux package manager found to install Docker");
    runShell(command);
    return;
  }
  if (process.platform === "win32") {
    if (!commandExists("winget")) throw new Error("winget is required to install Docker automatically on Windows");
    run("winget", ["install", "--id", "Docker.DockerDesktop", "-e", "--source", "winget"]);
    return;
  }
  throw new Error(`Unsupported platform for automatic Docker installation: ${process.platform}`);
}

function askYesNo(question) {
  if (!process.stdin.isTTY) return Promise.resolve(false);
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise((resolve) => {
    rl.question(`${question} [y/N] `, (answer) => {
      rl.close();
      resolve(answer.trim().toLowerCase() === "y" || answer.trim().toLowerCase() === "yes");
    });
  });
}

async function handleDockerCapability() {
  phase("Checking optional Docker capability");
  if (commandExists("docker")) {
    saveState({ docker: { status: "present" } });
    console.log("Docker command found. Container-backed workflows can use it when the daemon is running.");
    return;
  }

  const envChoice = process.env.ORNNLAB_INSTALL_DOCKER;
  const install = envChoice === "1" || (envChoice !== "0" && (await askYesNo("Docker is optional for first launch. Install Docker now?")));
  if (!install) {
    saveState({ docker: { status: "skipped" } });
    console.log("Docker install skipped. You can rerun `ORNNLAB_INSTALL_DOCKER=1 ornnlab install` later.");
    return;
  }

  try {
    installDocker();
    saveState({ docker: { status: commandExists("docker") ? "installed" : "installed_needs_restart" } });
  } catch (error) {
    saveState({ docker: { status: "failed", error: error.message } });
    console.warn(`Docker install failed: ${error.message}`);
    console.warn("Continuing because Docker is optional for first WebUI launch.");
  }
}

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
  saveState({ backend: { status: "ready" } });
}

function syncFrontendDependencies() {
  phase("Installing frontend dependencies");
  run("npm", ["ci"], { cwd: path.join(sourceDir, "frontend") });
  saveState({ frontend: { status: "ready" } });
}

async function setup() {
  try {
    phase("Checking required prerequisites");
    ensureRequiredTool("git");
    ensureRequiredTool("uv");
    ensureRequiredTool("node");
    ensureRequiredTool("npm");
    saveState({ prerequisites: { status: "ready", tools: ["git", "uv", "node", "npm"] } });

    await handleDockerCapability();
    ensureProjectSource();
    if (!backendReady()) syncBackendDependencies();
    if (!frontendReady()) syncFrontendDependencies();
    console.log(`\nOrnnLab source is ready at ${sourceDir}`);
  } catch (error) {
    saveState({ lastError: { message: error.message, command: process.argv.join(" ") } });
    throw error;
  }
}

async function ensureReady() {
  const missingProjectDeps = !sourceReady() || !backendReady() || !frontendReady();
  if (missingProjectDeps) {
    console.log("OrnnLab bootstrap is incomplete; running setup now.");
    await setup();
  }
  ensureSource();
}

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
  console.log("OrnnLab is starting.");
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

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command) {
    await runDev({ setupIfMissing: true });
    return;
  }
  if (command === "--help" || command === "-h" || command === "help") {
    console.log(help.trimEnd());
    return;
  }
  if (command === "--version" || command === "-V") {
    console.log(packageVersion);
    return;
  }
  if (command === "path") {
    console.log(sourceDir);
    return;
  }
  if (command === "install" || command === "setup") {
    await setup();
    return;
  }
  if (command === "web") {
    runBackend(args);
    return;
  }
  if (command === "ui") {
    runFrontend(args);
    return;
  }
  if (command === "doctor") {
    runDoctor(args);
    return;
  }
  if (command === "dev" || command === "start") {
    await runDev();
    return;
  }
  throw new Error(`Unknown command: ${command}`);
}

main().catch((error) => {
  console.error(error.message);
  console.error("Rerun `ornnlab install` after fixing the issue; bootstrap will retry incomplete stages.");
  process.exit(1);
});
