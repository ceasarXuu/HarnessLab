const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const testFiles = fs.readdirSync(__dirname)
  .filter((name) => name.endsWith(".test.js"))
  .map((name) => path.join(__dirname, name));
const result = spawnSync(process.execPath, ["--test", ...testFiles], { stdio: "inherit" });

process.exit(result.status ?? 1);
