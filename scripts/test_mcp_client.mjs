// Independent official SDK -> direct HTTP and stdio bridge -> Rust behavior; synthetic state only.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { createServer } from "node:http";
import { randomBytes } from "node:crypto";
import { resolve, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";
import {
  Client,
  StreamableHTTPClientTransport,
} from "../clients/nextcloud/node_modules/@modelcontextprotocol/client/dist/index.mjs";
import { StdioClientTransport } from "../clients/nextcloud/node_modules/@modelcontextprotocol/client/dist/stdio.mjs";
const root = dirname(dirname(fileURLToPath(import.meta.url)));
const temporary = await mkdtemp(join(tmpdir(), "mealshop-mcp-client-"));
const token = randomBytes(32).toString("hex"),
  password = randomBytes(24).toString("hex");
const reserve = createServer();
reserve.listen(0, "127.0.0.1");
await once(reserve, "listening");
const port = reserve.address().port;
await new Promise((r) => reserve.close(r));
const binary = resolve(
  process.argv[2] ?? join(root, "target/debug/mealshop-server"),
);
const server = spawn(binary, [], {
  cwd: root,
  env: {
    PATH: process.env.PATH,
    MEALSHOP_LISTEN_ADDR: `127.0.0.1:${port}`,
    MEALSHOP_RECIPE_DIR: join(temporary, "recipes"),
    MEALSHOP_HOUSEHOLD_DIR: join(temporary, "household"),
    MEALSHOP_HOUSEHOLD_TOKEN: token,
  },
  stdio: ["ignore", "pipe", "pipe"],
});
let gateway,
  clients = [];
const backend = `http://127.0.0.1:${port}`;
async function api(path, method = "GET", value) {
  const response = await fetch(backend + path, {
    method,
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: value === undefined ? undefined : JSON.stringify(value),
  });
  assert.equal(
    response.ok,
    true,
    `Synthetic API ${path} failed: ${response.status}`,
  );
  return response.json();
}
async function client(transport, mode) {
  const c = new Client(
    { name: "Mealshop acceptance client", version: "1" },
    { versionNegotiation: { mode } },
  );
  clients.push(c);
  await c.connect(transport);
  return c;
}
const recipe = {
  title: "MCP SDK acceptance recipe",
  language: "en",
  servings: 2,
  preparation: "as_recorded",
  lines: [
    { ingredient_id: "oats", quantity: { amount: "0.100500", unit: "kg" } },
  ],
  steps: [],
};
try {
  let ready = false;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (server.exitCode !== null)
      throw new Error("Synthetic backend stopped before readiness");
    try {
      if ((await fetch(backend + "/health")).ok) {
        ready = true;
        break;
      }
    } catch {}
    await new Promise((r) => setTimeout(r, 100));
  }
  assert.equal(ready, true, "Synthetic backend did not start");
  await api("/api/household/mcp/settings", "PUT", {
    expected_revision: 0,
    grants: { local: "create" },
  });
  const direct = await client(
    new StreamableHTTPClientTransport(new URL(backend + "/api/household/mcp"), {
      requestInit: { headers: { Authorization: `Bearer ${token}` } },
    }),
    "auto",
  );
  const listed = await direct.listTools();
  assert.equal(listed.tools.length, 6);
  const preview = await direct.callTool({
    name: "recipe_preview_create",
    arguments: { recipe },
  });
  assert.equal(preview.isError, false);
  assert.equal(preview.structuredContent.saved, false);
  const input = {
    preview_id: preview.structuredContent.preview_id,
    idempotency_key: "synthetic-sdk-request-0001",
    confirmed: true,
  };
  const saved = await direct.callTool({
    name: "recipe_create",
    arguments: input,
  });
  assert.equal(saved.isError, false);
  const id = saved.structuredContent.record.id;
  const repeat = await direct.callTool({
    name: "recipe_create",
    arguments: input,
  });
  assert.equal(repeat.structuredContent.replayed, true);
  assert.equal(repeat.structuredContent.record.id, id);
  // This synthetic gateway models only Nextcloud's verified Basic -> authenticated-domain hop.
  gateway = createServer(async (req, res) => {
    try {
      if (
        req.headers.authorization !==
        "Basic " + Buffer.from(`synthetic:${password}`).toString("base64")
      ) {
        res.writeHead(401);
        res.end();
        return;
      }
      if (
        req.headers["x-mealshop-request"] !== "1" ||
        req.url !== "/api/household/mcp"
      ) {
        res.writeHead(403);
        res.end();
        return;
      }
      const chunks = [];
      for await (const chunk of req) chunks.push(chunk);
      const headers = { ...req.headers, authorization: `Bearer ${token}` };
      delete headers.host;
      delete headers["content-length"];
      delete headers.connection;
      const response = await fetch(backend + "/api/household/mcp", {
        method: "POST",
        headers,
        body: Buffer.concat(chunks),
      });

      res.writeHead(response.status, { "Content-Type": "application/json" });
      res.end(Buffer.from(await response.arrayBuffer()));
    } catch {
      res.writeHead(502);
      res.end();
    }
  });
  gateway.listen(0, "127.0.0.1");
  await once(gateway, "listening");
  const url = `http://127.0.0.1:${gateway.address().port}/api/household/mcp`;
  for (const mode of ["auto", "legacy", { pin: "2026-07-28" }]) {
    const c = await client(
      new StdioClientTransport({
        command: "python3",
        args: [join(root, "scripts/mcp_stdio.py")],
        env: {
          MEALSHOP_MCP_URL: url,
          MEALSHOP_MCP_USER: "synthetic",
          MEALSHOP_MCP_APP_PASSWORD: password,
        },
        stderr: "pipe",
      }),
      mode,
    );
    const tools = await c.listTools();
    assert.equal(tools.tools.length, 6);
    const read = await c.callTool({
      name: "recipe_get",
      arguments: { id, revision: 1 },
    });
    assert.equal(
      read.structuredContent.content.recipe.lines[0].grams,
      "100.500000",
    );
    const retry = await c.callTool({ name: "recipe_create", arguments: input });
    assert.equal(retry.structuredContent.record.id, id);
    assert.equal(retry.structuredContent.replayed, true);
    await c.close();
  }
  const activity = await api("/api/household/mcp/activity");
  assert.equal(activity.total, 1);
  assert.equal(activity.entries[0].actor, "local");
  const imported = {
    ...recipe,
    origin: {
      kind: "imported",
      source_url: "https://example.org/sdk-original",
    },
  };
  await api(`/api/recipes/${id}`, "PUT", {
    expected_revision: 1,
    recipe: imported,
  });
  const restricted = await direct.callTool({
    name: "recipe_get",
    arguments: { id, revision: 1 },
  });
  assert.deepEqual(restricted.structuredContent.content, {
    kind: "source_link",
    source_url: "https://example.org/sdk-original",
  });
  await api("/api/household/mcp/settings", "PUT", {
    expected_revision: 1,
    grants: { local: "read" },
  });
  const forbidden = await direct.callTool({
    name: "recipe_create",
    arguments: input,
  });
  assert.equal(forbidden.isError, true);
  assert.equal(forbidden.structuredContent.code, "mcp_read_only");
  console.log(
    "PASS: official MCP SDK 2.4 direct modern HTTP and modern/legacy stdio; exact quantities, confirmed creation, retry identity, audit, retroactive source-only reads and scope reduction",
  );
} finally {
  await Promise.allSettled(clients.map((c) => c.close()));
  if (gateway) {
    gateway.closeAllConnections();
    await new Promise((r) => gateway.close(r));
  }
  server.kill("SIGTERM");
  await Promise.race([
    once(server, "exit"),
    new Promise((r) => setTimeout(r, 5000)),
  ]);
  if (server.exitCode === null) {
    server.kill("SIGKILL");
    await once(server, "exit");
  }
  await rm(temporary, { recursive: true, force: true });
}
