// Reuse the product's typed recipe/quantity contracts; Rust remains the semantic validator.
import { build } from "../clients/nextcloud/node_modules/esbuild/lib/main.js";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
const root = dirname(dirname(fileURLToPath(import.meta.url)));
const bundle = await build({
  stdin: {
    contents:
      "export {recipeSchema} from './clients/nextcloud/src/contracts'; export {z} from './clients/nextcloud/node_modules/zod/index.js'",
    resolveDir: root,
  },
  bundle: true,
  write: false,
  platform: "node",
  format: "cjs",
  logLevel: "silent",
});
const module = { exports: {} };
// The bundled source is our committed contracts, never a client-provided schema or recipe.
new Function("module", "exports", "require", bundle.outputFiles[0].text)(
  module,
  module.exports,
  createRequire(import.meta.url),
);
const { recipeSchema, z } = module.exports;
const schema = z.toJSONSchema(recipeSchema, {
  io: "input",
  target: "draft-2020-12",
});
schema.properties.author.maxLength = 200;
schema.required = schema.required.filter((k) => k !== "author");
schema.properties.title.minLength = 1;
schema.properties.title.maxLength = 200;
schema.properties.servings.maximum = 1000000;
schema.properties.parts.maxItems = 30;
schema.properties.steps.maxItems = 100;
schema.properties.lines.minItems = 1;
schema.properties.lines.maxItems = 100;
const line = schema.properties.lines.items;
line.required = line.required.filter((k) => k !== "grams");
line.anyOf = [{ required: ["grams"] }, { required: ["quantity"] }];
line.properties.grams.pattern = "^\\d+(?:\\.\\d{1,6})?$";
for (const key of [
  "active_seconds",
  "passive_seconds",
  "passive_max_seconds",
]) {
  schema.properties.steps.items.properties[key].minimum = 0;
  schema.properties.steps.items.properties[key].maximum = 604800;
}
schema.properties.steps.items.properties.instruction.minLength = 1;
schema.properties.steps.items.properties.instruction.maxLength = 4000;
const target = join(root, "data/mcp-recipe-schema.json"),
  text = JSON.stringify(schema, null, 2) + "\n";
if (process.argv.includes("--check")) {
  if (readFileSync(target, "utf8") !== text)
    throw new Error(
      "Run node scripts/generate_mcp_schema.mjs and commit the MCP recipe schema",
    );
} else writeFileSync(target, text);
