const { phase } = require("./common");
const { saveState, sourceDir } = require("./state");
const { ensureRequiredTool } = require("./prerequisites");
const { handleDockerCapability } = require("./docker");
const {
  ensureProjectSource,
  ensureSource,
  sourceReady,
  backendReady,
  frontendReady,
  syncBackendDependencies,
  syncFrontendDependencies,
} = require("./source");

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

module.exports = { setup, ensureReady };
