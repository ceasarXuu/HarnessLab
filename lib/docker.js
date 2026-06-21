const readline = require("node:readline");
const { commandExists, run, runShell, phase, linuxInstaller } = require("./common");
const { saveState } = require("./state");
const { installHomebrewIfNeeded } = require("./prerequisites");

function installDocker() {
  phase("Installing optional Docker capability");
  if (process.platform === "darwin") {
    installHomebrewIfNeeded();
    run("brew", ["install", "docker", "colima"], { announce: true });
    console.log("Installed Docker CLI and Colima. Start the lightweight runtime later with: colima start");
    return;
  }
  if (process.platform === "linux") {
    const command = linuxInstaller("docker.io");
    if (!command) throw new Error("No supported Linux package manager found to install Docker");
    runShell(command, { announce: true });
    return;
  }
  if (process.platform === "win32") {
    throw new Error("Core-only Docker install is not automated on Windows. Use Docker Engine inside WSL; Docker Desktop is intentionally not installed by OrnnLab.");
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

module.exports = {
  installDocker,
  askYesNo,
  handleDockerCapability,
};
