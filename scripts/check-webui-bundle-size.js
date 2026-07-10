const fs = require("node:fs");
const path = require("node:path");

const assetsDir = path.resolve(__dirname, "../frontend/dist/assets");
const limits = { ".css": 50 * 1024, ".js": 400 * 1024 };
const files = fs.readdirSync(assetsDir).map((name) => path.join(assetsDir, name));

for (const extension of Object.keys(limits)) {
  const matching = files.filter((file) => path.extname(file) === extension);
  if (matching.length === 0) throw new Error(`No ${extension} asset was emitted.`);
  const largest = Math.max(...matching.map((file) => fs.statSync(file).size));
  if (largest > limits[extension]) {
    throw new Error(`Largest ${extension} asset is ${largest} bytes; limit is ${limits[extension]} bytes.`);
  }
}
