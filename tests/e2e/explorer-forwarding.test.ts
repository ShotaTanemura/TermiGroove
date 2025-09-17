import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Explorer shows help line; Up/Down key smoke does not crash", async ({ terminal }) => {
  // Match a stable substring in the explorer's help line
  await expect(
    terminal.getByText("Enter: to pads", { full: false })
  ).toBeVisible();

  // Send a couple of navigation keys; nothing should crash.
  terminal.keyDown(2);
  terminal.keyUp(1);

  // Still alive and help line still visible (substring)
  await expect(
    terminal.getByText("Space: select", { full: false })
  ).toBeVisible();
});


