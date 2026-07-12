const http = require("node:http");

const args = new Map();
for (let index = 2; index < process.argv.length; index += 1) {
  const key = process.argv[index];
  if (!key.startsWith("--")) continue;
  const next = process.argv[index + 1];
  if (!next || next.startsWith("--")) args.set(key, "true");
  else {
    args.set(key, next);
    index += 1;
  }
}

const port = Number(args.get("--port"));
const host = args.get("--host") || "127.0.0.1";
const role = args.get("--role") || "service";
const delayHealthMs = Number(args.get("--delay-health") || 0);
const failAfterFirst = args.get("--fail-after-first");
const printSecret = args.get("--print-secret");
const splitSecret = args.get("--split-secret");
const printAuthorization = args.get("--print-authorization");
const ignoreSigterm = args.has("--ignore-sigterm");
if (!Number.isInteger(port)) {
  console.error("missing --port");
  process.exit(2);
}
if (failAfterFirst) {
  if (require("node:fs").existsSync(failAfterFirst)) process.exit(42);
  require("node:fs").writeFileSync(failAfterFirst, String(process.pid));
}
if (printSecret) {
  console.log(`ANTHROPIC_API_KEY=${printSecret}`);
}
if (printAuthorization) {
  console.log(`Authorization: Bearer ${printAuthorization}`);
}
if (splitSecret) {
  process.stdout.write("ANTHROPIC_API_KEY=");
  setTimeout(() => process.stdout.write(`${splitSecret}\n`), 20);
}

const readyAt = Date.now() + delayHealthMs;

const server = http.createServer((request, response) => {
  if (request.url === "/api/webui/v1/system/health") {
    if (Date.now() < readyAt) {
      response.writeHead(503, { "content-type": "application/json" });
      response.end(JSON.stringify({ error: { message: "starting" } }));
      return;
    }
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ data: { items: [{ component: role }] }, error: null }));
    return;
  }
  response.writeHead(404, { "content-type": "text/plain" });
  response.end("not found");
});

const listenDelayMs = splitSecret ? 50 : 0;
setTimeout(() => {
  server.listen(port, host, () => {
    console.log(`${role} ready ${host}:${port}`);
  });
}, listenDelayMs);

function shutdown() {
  server.close(() => process.exit(0));
}

process.on("SIGINT", shutdown);
process.on("SIGTERM", () => {
  if (!ignoreSigterm) shutdown();
});
