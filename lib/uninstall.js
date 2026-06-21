const fs = require("node:fs");
const path = require("node:path");
const { phase, askYesNo } = require("./common");
const { loadState, launcherDir, sourceDir, statePath } = require("./state");

function formatTimestamp(d = new Date()) {
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hours = String(d.getHours()).padStart(2, "0");
  const minutes = String(d.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day}-${hours}${minutes}`;
}

function safeExists(target) {
  try {
    return fs.existsSync(target);
  } catch {
    return false;
  }
}

async function handleUninstall(args = []) {
  const dryRun = args.includes("--dry-run") || args.includes("--check");
  const force = args.includes("--force");
  const state = loadState();

  const ornnlabRoot = path.dirname(launcherDir);
  const dataDir = path.join(ornnlabRoot, "data");
  const launcherExists = safeExists(launcherDir);
  const sourceExists = safeExists(sourceDir);
  const stateExists = safeExists(statePath);
  const dataExists = safeExists(dataDir);

  phase("OrnnLab uninstall plan");
  console.log("  Launcher-managed artifacts:");
  console.log(`    ${launcherDir}             - ${launcherExists ? "exists" : "not present"}`);
  console.log(`      ${sourceDir}   - ${sourceExists ? "exists" : "not present"}`);
  console.log(`      ${statePath} - ${stateExists ? "exists" : "not present"}`);
  console.log("");
  console.log("  User data (preserved by default):");
  console.log(`    ${dataDir}                 - ${dataExists ? "exists" : "not present"}`);
  console.log("");
  console.log("  Manual cleanup items (outside launcher ownership):");
  console.log("    npm uninstall -g ornnlab   (remove global launcher package)");
  console.log("    Docker, uv, Node.js        (installed by user or system, not removed)");
  console.log("");

  if (dryRun) {
    console.log("Dry-run mode: no changes made.");
    return;
  }

  let backupData = false;
  if (dataExists) {
    console.log("User experiment data exists at ~/.ornnlab/data.");
    console.log("By default, it will be left in place.");
    backupData = await askYesNo("Also move user data to the backup? (Data is moved, not deleted.)");
  }

  const confirmed = force || await askYesNo("Proceed with uninstall?");
  if (!confirmed) {
    console.log("Uninstall cancelled.");
    return;
  }

  const backupDir = path.join(ornnlabRoot, `backup-${formatTimestamp()}`);
  phase("Moving launcher-managed artifacts to backup");
  console.log(`  Backup location: ${backupDir}`);
  fs.mkdirSync(backupDir, { recursive: true });

  const uninstallRecord = {
    performedAt: new Date().toISOString(),
    backupDir,
    dataPreserved: !backupData,
    previousState: state,
  };
  fs.writeFileSync(
    path.join(backupDir, "uninstall-record.json"),
    `${JSON.stringify(uninstallRecord, null, 2)}\n`,
  );

  if (launcherExists) {
    const launcherBackup = path.join(backupDir, "launcher");
    console.log(`  Moving ${launcherDir} -> ${launcherBackup}`);
    fs.renameSync(launcherDir, launcherBackup);
  }

  if (backupData && dataExists) {
    const dataBackup = path.join(backupDir, "data");
    console.log(`  Moving ${dataDir} -> ${dataBackup}`);
    fs.renameSync(dataDir, dataBackup);
  }

  phase("Uninstall complete");
  console.log(`  Backup location: ${backupDir}`);
  console.log("");
  if (!backupData && dataExists) {
    console.log(`  User data preserved at: ${dataDir}`);
  }
  console.log("");
  console.log("Remaining manual cleanup:");
  console.log("  1. npm uninstall -g ornnlab  (remove global launcher package)");
  console.log("  2. Optionally remove Docker, uv, or Node.js if no longer needed.");
  console.log("");
  console.log("No files were irreversibly deleted. All moved items are in the backup directory.");
}

module.exports = { handleUninstall };
