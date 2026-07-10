const { spawnSync } = require("node:child_process");
const path = require("node:path");

const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const result = spawnSync(npm, ["run", "build"], {
  cwd: path.resolve(__dirname, "../frontend"),
  encoding: "utf-8",
  env: { ...process.env, VITE_ORNNLAB_DATA_MODE: "mock" },
  shell: process.platform === "win32",
});
const output = `${result.stdout || ""}${result.stderr || ""}`;

if (result.status === 0 || !output.includes("Production WebUI builds require")) {
  throw new Error(`Production build accepted mock mode:\n${output}`);
}
