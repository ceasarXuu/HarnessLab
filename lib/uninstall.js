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

// B3 修复：检测 sourceDir 是否在 launcherDir 内
// 返回 true 表示 sourceDir 在 launcherDir 之外（独立外部资源）
// 返回 false 表示 sourceDir 等于或位于 launcherDir 内（随 launcher 一起移动）
function isExternalSource() {
  const launcherAbs = path.resolve(launcherDir);
  const sourceAbs = path.resolve(sourceDir);
  if (launcherAbs === sourceAbs) {
    // 病态配置：sourceDir === launcherDir。视为内部，不重复显示。
    return false;
  }
  const relative = path.relative(launcherAbs, sourceAbs);
  // sourceDir 在 launcherDir 之外当且仅当 relative 以 .. 开头或是绝对路径
  return relative.startsWith("..") || path.isAbsolute(relative);
}

// B2 修复：原子化移动 — 失败时清理 backupDir 残留物
function atomicMoveToBackup(backupDir, sourcePath, destName) {
  const dest = path.join(backupDir, destName);
  try {
    fs.renameSync(sourcePath, dest);
    return dest;
  } catch (error) {
    // 跨设备 rename 失败：尝试递归复制 + 删除
    if (error.code === "EXDEV") {
      try {
        fs.cpSync(sourcePath, dest, { recursive: true, errorOnExist: true, force: false });
        fs.rmSync(sourcePath, { recursive: true, force: true });
        return dest;
      } catch (cpError) {
        // 复制失败：清理 dest 残留
        try {
          fs.rmSync(dest, { recursive: true, force: true });
        } catch {
          // ignore cleanup errors
        }
        throw new Error(`Failed to move ${sourcePath} to ${dest}: ${cpError.message}`);
      }
    }
    throw error;
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
  const externalSource = isExternalSource();

  phase("OrnnLab uninstall plan");
  console.log("  Launcher-managed artifacts (will be moved to backup):");
  console.log(`    ${launcherDir}             - ${launcherExists ? "exists" : "not present"}`);
  if (!externalSource) {
    console.log(`      ${sourceDir}   - ${sourceExists ? "exists (inside launcher, will move with it)" : "not present"}`);
  }
  console.log(`      ${statePath} - ${stateExists ? "exists" : "not present"}`);
  console.log("");
  if (externalSource) {
    console.log("  External resources (NOT affected by uninstall):");
    console.log(`    ${sourceDir}   - ${sourceExists ? "exists (external, will be left untouched)" : "not present"}`);
    console.log("");
  }
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

  const movedItems = [];
  let recordWritten = false;

  try {
    if (launcherExists) {
      console.log(`  Moving ${launcherDir} -> ${path.join(backupDir, "launcher")}`);
      atomicMoveToBackup(backupDir, launcherDir, "launcher");
      movedItems.push({ from: launcherDir, to: path.join(backupDir, "launcher") });
    }

    if (backupData && dataExists) {
      console.log(`  Moving ${dataDir} -> ${path.join(backupDir, "data")}`);
      atomicMoveToBackup(backupDir, dataDir, "data");
      movedItems.push({ from: dataDir, to: path.join(backupDir, "data") });
    }

    // B2 修复：record 在所有 move 成功后才写入，确保 backupDir 不会留下空目录+误导性 record
    const uninstallRecord = {
      performedAt: new Date().toISOString(),
      backupDir,
      dataPreserved: !backupData,
      externalSource,
      externalSourcePath: externalSource ? sourceDir : null,
      movedItems,
      previousState: state,
    };
    fs.writeFileSync(
      path.join(backupDir, "uninstall-record.json"),
      `${JSON.stringify(uninstallRecord, null, 2)}\n`,
    );
    recordWritten = true;
  } catch (error) {
    // B2 修复：失败时清理 backupDir（如果没有成功移入任何东西且没有写入 record）
    if (movedItems.length === 0 && !recordWritten) {
      try {
        fs.rmdirSync(backupDir);
        console.error(`Cleaned up empty backup directory: ${backupDir}`);
      } catch {
        // ignore cleanup errors
      }
    } else {
      console.error(`Partial uninstall: ${movedItems.length} item(s) moved before failure.`);
      console.error(`Backup directory preserved for inspection: ${backupDir}`);
    }
    throw error;
  }

  phase("Uninstall complete");
  console.log(`  Backup location: ${backupDir}`);
  console.log("");
  if (!backupData && dataExists) {
    console.log(`  User data preserved at: ${dataDir}`);
  }
  if (externalSource && sourceExists) {
    console.log(`  External source preserved at: ${sourceDir}`);
  }
  console.log("");
  console.log("Remaining manual cleanup:");
  console.log("  1. npm uninstall -g ornnlab  (remove global launcher package)");
  console.log("  2. Optionally remove Docker, uv, or Node.js if no longer needed.");
  console.log("");
  console.log("No files were irreversibly deleted. All moved items are in the backup directory.");
}

module.exports = { handleUninstall };
