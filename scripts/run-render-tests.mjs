import { readdirSync } from "node:fs";
import { basename, join } from "node:path";
import { spawnSync } from "node:child_process";

const testsDir = "tests";
const renderTests = readdirSync(testsDir)
  .filter((file) => file.endsWith("_render_test.rs"))
  .map((file) => basename(file, ".rs"))
  .sort();

if (renderTests.length === 0) {
  console.error("No render test files found in tests/");
  process.exit(1);
}

for (const testName of renderTests) {
  const listResult = spawnSync(
    "cargo",
    ["test", "--test", testName, "--", "--list"],
    {
      encoding: "utf8",
      cwd: process.cwd(),
    }
  );

  if (listResult.status !== 0) {
    process.stderr.write(listResult.stderr ?? "");
    process.exit(listResult.status ?? 1);
  }

  const cases = (listResult.stdout ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.replace(/: test$/, ""));

  for (const caseName of cases) {
    console.log(`\n==> Running ${testName} :: ${caseName}`);
    const result = spawnSync(
      "cargo",
      [
        "test",
        "--test",
        testName,
        caseName,
        "--",
        "--exact",
        "--test-threads=1",
      ],
      {
        stdio: "inherit",
        cwd: process.cwd(),
      }
    );

    if (result.status !== 0) {
      process.exit(result.status ?? 1);
    }
  }
}

console.log("\nRender test suite completed.");
