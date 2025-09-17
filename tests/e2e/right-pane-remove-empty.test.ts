import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Right pane: removal keys on empty list do not change status", async ({ terminal }) => {
  // Initial status
  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();
  // Focus Right
  terminal.write("\t");
  await new Promise((r) => setTimeout(r, 50));
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();

  // Press removal keys on empty list
  terminal.write(" ");
  await new Promise((r) => setTimeout(r, 50));
  // Delete: send escape sequence for DEL (DEL often not needed; smoke by writing \x7f)
  terminal.write("\x7f");
  await new Promise((r) => setTimeout(r, 50));
  terminal.write("d");
  await new Promise((r) => setTimeout(r, 50));

  // Status should remain either Right focus or Ready (no change to removal messages)
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();
});


