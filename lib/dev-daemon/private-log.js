const fs = require("node:fs");

function openPrivateLog(logPath, ensureHome) {
  ensureHome();
  const fd = fs.openSync(logPath, "a", 0o600);
  fs.chmodSync(logPath, 0o600);
  return fd;
}

function attachSanitizedLog(child, logPath, ensureHome) {
  ensureHome();
  fs.closeSync(fs.openSync(logPath, "a", 0o600));
  fs.chmodSync(logPath, 0o600);
  const stream = fs.createWriteStream(logPath, { flags: "a", mode: 0o600 });
  const buffers = new Map();
  const write = (source, chunk) => {
    const next = `${buffers.get(source) || ""}${String(chunk)}`;
    const lines = next.split(/\r?\n/);
    buffers.set(source, lines.pop() || "");
    for (const line of lines) stream.write(`${redactSecrets(line)}\n`);
  };
  child.stdout.on("data", (chunk) => write("stdout", chunk));
  child.stderr.on("data", (chunk) => write("stderr", chunk));
  child.once("close", () => {
    for (const buffered of buffers.values()) {
      if (buffered) stream.write(redactSecrets(buffered));
    }
    stream.end();
  });
}

function redactSecrets(text) {
  return text
    .replace(/((?:API[_-]?KEY|TOKEN|PASSWORD|SECRET)[A-Z0-9_-]*=)[^\s]+/gi, "$1[REDACTED]")
    .replace(/(Authorization:\s*Bearer\s+)[^\s]+/gi, "$1[REDACTED]")
    .replace(/(["']?(?:api[_-]?key|token|password|secret)["']?\s*[:=]\s*["']?)[^"',}\s]+/gi, "$1[REDACTED]");
}

module.exports = {
  attachSanitizedLog,
  openPrivateLog,
  redactSecrets,
};
