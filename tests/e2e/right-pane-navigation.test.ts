import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Right pane: Tab focus, Up/Down keep focus and stay stable", async ({ terminal }) => {
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();

  // Focus Right
  terminal.write("\t");
  await new Promise((r) => setTimeout(r, 50));
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();

  // Up/Down should be no-ops on empty list and keep focus/status
  terminal.keyUp(1);
  terminal.keyDown(1);
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();

  // Title remains visible
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();
});



