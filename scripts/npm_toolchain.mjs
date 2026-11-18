import { readFileSync } from "node:fs";

const { packageManager } = JSON.parse(
  readFileSync(
    new URL("../clients/nextcloud/package.json", import.meta.url),
    "utf8",
  ),
);
if (
  typeof packageManager !== "string" ||
  !/^npm@\d+\.\d+\.\d+$/.test(packageManager)
) {
  console.error("packageManager must pin npm exactly (npm@X.Y.Z)");
  process.exit(1);
}
process.stdout.write(packageManager);
