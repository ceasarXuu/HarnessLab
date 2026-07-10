const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const readline = require("node:readline");

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

function phase(message) {
  console.log(`\n==> ${message}`);
}

function run(command, args, options = {}) {
  if (options.announce) {
    console.log(`$ ${command} ${args.join(" ")}`.trimEnd());
  }
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: options.env || process.env,
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
  if (options.announce) {
    console.log(`$ ${command}`);
  }
  return run(command, [], { ...options, shell: true, announce: false });
}

function runCapture(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    env: process.env,
    shell: options.shell || false,
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf-8",
  });
  if (result.error) {
    throw new Error(`${command} failed to start: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}: ${result.stderr}`);
  }
  return result.stdout.trim();
}

function spawnAttached(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: options.cwd,
    detached: options.detached || false,
    env: options.env || process.env,
    shell: options.shell ?? process.platform === "win32",
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

module.exports = {
  addPathIfPresent,
  refreshBootstrapPath,
  phase,
  run,
  runShell,
  runCapture,
  spawnAttached,
  commandExists,
  askYesNo,
  sudoPrefix,
  linuxInstaller,
};
