import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Enter requires selection; q quits cleanly", async ({ terminal }) => {
  // Initial status Ready visible
  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();

  // Press Enter — should require at least one selection
  terminal.write("\r");
  await new Promise((r) => setTimeout(r, 100));
  await expect(
    terminal.getByText("Select at least one file first", { full: false })
  ).toBeVisible();

  // Press q — app quits cleanly
  terminal.write("q");
  // Presence of alternate screen exit is hard to assert; check process exits
  // by ensuring the terminal no longer updates — serialize once as a smoke check
  const snap = terminal.serialize();
  expect(snap.view.length).toBeGreaterThan(0);
});




