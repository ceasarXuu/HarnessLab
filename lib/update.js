const path = require("node:path");
const { phase, run, askYesNo, runCapture } = require("./common");
const { loadState, saveState, sourceDir, packageVersion } = require("./state");
const { ensureSource, sourceReady, backendReady, frontendReady } = require("./source");

async function handleUpdate(args = []) {
  const dryRun = args.includes("--dry-run") || args.includes("--check");
  const state = loadState();
  const previousVersion = state.launcherVersion || packageVersion;

  // B1 修复：检测源码缺失。update 假设 install 已执行；缺失时拒绝运行，
  // 避免 npm install 成功而 source 缺失造成的部分更新状态。
  if (!sourceReady()) {
    console.error("ERROR: OrnnLab source checkout not found at:");
    console.error(`  ${sourceDir}`);
    console.error("");
    console.error("`ornnlab update` requires an existing install. Run `ornnlab install` first.");
    process.exitCode = 1;
    return;
  }

  phase("OrnnLab update plan");
  console.log(`  Current launcher version: ${previousVersion}`);
  console.log(`  Source checkout: ${sourceDir}`);
  console.log("");
  console.log("  Planned actions:");
  console.log("    1. npm install -g ornnlab@latest  (update global launcher)");
  console.log("    2. git pull --ff-only              (update source checkout)");
  console.log("    3. uv sync --group dev             (sync backend dependencies)");
  console.log("    4. npm ci                          (sync frontend dependencies)");
  console.log("    5. Verify Harbor + OrnnLab imports");
  console.log("");

  if (dryRun) {
    console.log("Dry-run mode: no changes made.");
    return;
  }

  const confirmed = await askYesNo("Proceed with update?");
  if (!confirmed) {
    console.log("Update cancelled.");
    return;
  }

  phase("Updating global npm launcher");
  try {
    run("npm", ["install", "-g", "ornnlab@latest"], { announce: true });
  } catch (error) {
    console.warn(`Global launcher update failed: ${error.message}`);
    console.warn("Continuing with source and dependency update.");
  }

  phase("Updating source checkout");
  ensureSource();
  run("git", ["pull", "--ff-only"], { cwd: sourceDir, announce: true });

  phase("Syncing backend dependencies");
  run("uv", ["sync", "--group", "dev"], { cwd: sourceDir, announce: true });

  phase("Syncing frontend dependencies");
  run("npm", ["ci"], { cwd: path.join(sourceDir, "frontend"), announce: true });

  phase("Verifying update");
  run("uv", ["run", "python", "-c", "import harbor; import ornnlab"], { cwd: sourceDir });
  run("uv", ["run", "ornnlab", "--version"], { cwd: sourceDir, stdio: "ignore" });

  let newVersion = packageVersion;
  try {
    newVersion = runCapture("npm", ["view", "ornnlab", "version"]);
  } catch {
    // npm view 失败时使用当前版本
  }

  saveState({
    update: {
      previousVersion,
      newVersion,
      performedAt: new Date().toISOString(),
    },
    backend: { status: "ready", verified: true },
    source: { status: "ready", path: sourceDir },
  });

  phase("Update complete");
  console.log(`  Previous version: ${previousVersion}`);
  console.log(`  Updated to:       ${newVersion}`);
  console.log(`  Source:           ${sourceDir}`);
  console.log("");
  console.log("User data and local run artifacts were preserved.");
  console.log("If the global launcher was updated, run `ornnlab --version` to confirm.");
}

module.exports = { handleUpdate };
