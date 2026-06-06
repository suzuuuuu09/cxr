import { expect, test } from "bun:test";

test("cargo test passes", () => {
  const result = Bun.spawnSync(["cargo", "test", "--locked"], {
    stdout: "inherit",
    stderr: "inherit",
  });

  expect(result.exitCode).toBe(0);
});
