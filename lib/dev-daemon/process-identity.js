const crypto = require("node:crypto");
const { spawnSync } = require("node:child_process");

function createProcessToken() {
  return crypto.randomBytes(24).toString("hex");
}

function isPidAlive(pid) {
  if (!pid) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function isManagedProcessAlive(pid, token) {
  if (!isPidAlive(pid) || !token) return false;
  return processCommand(pid).includes(token);
}

function processCommand(pid) {
  if (!pid || !isPidAlive(pid)) return "";
  if (process.platform === "win32") return powershellOutput(
    `(Get-CimInstance Win32_Process -Filter "ProcessId=${pid}").CommandLine`
  );
  const result = spawnSync("ps", ["-p", String(pid), "-o", "command="], { encoding: "utf8" });
  return result.status === 0 ? result.stdout : "";
}

function processStartTime(pid) {
  if (!pid || !isPidAlive(pid)) return null;
  if (process.platform === "win32") {
    return powershellOutput(
      `(Get-CimInstance Win32_Process -Filter "ProcessId=${pid}").CreationDate`
    ) || null;
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "lstart="], { encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() || null : null;
}

function powershellOutput(script) {
  const result = spawnSync(
    "powershell",
    ["-NoProfile", "-NonInteractive", "-Command", script],
    { encoding: "utf8", windowsHide: true }
  );
  return result.status === 0 ? result.stdout.trim() : "";
}

module.exports = {
  createProcessToken,
  isManagedProcessAlive,
  isPidAlive,
  processCommand,
  processStartTime,
};

