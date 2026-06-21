const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { version: packageVersion } = require("../package.json");

const launcherDir = process.env.ORNNLAB_LAUNCHER_HOME || path.join(os.homedir(), ".ornnlab", "launcher");
const sourceDir = process.env.ORNNLAB_SOURCE || path.join(launcherDir, "source");
const statePath = path.join(launcherDir, "bootstrap-state.json");
const stateSchemaVersion = 1;

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
    schemaVersion: stateSchemaVersion,
    launcherVersion: packageVersion,
    updatedAt: new Date().toISOString(),
  };
  fs.writeFileSync(statePath, `${JSON.stringify(state, null, 2)}\n`);
}

module.exports = {
  packageVersion,
  launcherDir,
  sourceDir,
  statePath,
  stateSchemaVersion,
  loadState,
  saveState,
};
