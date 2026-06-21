const { commandExists, run, runShell, phase, refreshBootstrapPath, linuxInstaller } = require("./common");

function installHomebrewIfNeeded() {
  if (commandExists("brew")) return;
  phase("Installing Homebrew because no supported macOS package manager was found");
  runShell('/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"', {
    announce: true,
  });
}

function installRequiredTool(tool) {
  if (process.env.ORNNLAB_AUTO_INSTALL === "0") {
    throw new Error(`Required command not available on PATH: ${tool}`);
  }

  phase(`Installing required tool: ${tool}`);
  if (process.platform === "darwin") {
    installHomebrewIfNeeded();
    const brewName = tool === "npm" || tool === "node" ? "node" : tool;
    run("brew", ["install", brewName], { announce: true });
    return;
  }

  if (process.platform === "linux") {
    if (tool === "uv") {
      runShell('curl -LsSf https://astral.sh/uv/install.sh | sh', { announce: true });
      refreshBootstrapPath();
      return;
    }
    const packageName = tool === "npm" || tool === "node" ? "nodejs npm" : tool;
    const command = linuxInstaller(packageName);
    if (!command) {
      throw new Error(`No supported Linux package manager found to install ${tool}`);
    }
    runShell(command, { announce: true });
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
    run("winget", ["install", "--id", ids[tool], "-e", "--source", "winget"], { announce: true });
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

module.exports = {
  installHomebrewIfNeeded,
  installRequiredTool,
  ensureRequiredTool,
};
