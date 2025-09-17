import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Acceptance: initial, focus toggle, bounds, enter requires selection, quit", async ({ terminal }) => {
  // Initial render
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();
  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();

  // Focus toggle
  terminal.write("\t");
  await new Promise((r) => setTimeout(r, 50));
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();

  // Right bounds (no items yet -> no-ops, but still stable)
  terminal.keyUp(1);
  terminal.keyDown(1);
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();

  // Enter requires selection
  terminal.write("\r");
  await new Promise((r) => setTimeout(r, 50));
  await expect(terminal.getByText("Select at least one file first", { full: false })).toBeVisible();

  // Quit
  terminal.write("q");
  const snap = terminal.serialize();
  expect(snap.view.length).toBeGreaterThan(0);
});


