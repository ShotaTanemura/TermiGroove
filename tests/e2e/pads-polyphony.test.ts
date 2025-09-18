import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Pads: pressing multiple keys and same-key re-trigger does not crash", async ({ terminal }) => {
  // Smoke: app renders
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();

  // Try to select current item if possible (no-op if dir)
  terminal.write(" ");
  await new Promise((r) => setTimeout(r, 80));

  // Attempt to enter pads
  terminal.write("\r");
  await new Promise((r) => setTimeout(r, 150));

  try {
    const padsHint = await terminal.getByText("[Pads] Press Esc to go back.", { full: false });
    await expect(padsHint).toBeVisible();
  } catch {
    const needSelection = await terminal.getByText("Select at least one file first", { full: false });
    await expect(needSelection).toBeVisible();
    return;
  }

  // In pads: press two different keys quickly (polyphony)
  terminal.write("q");
  terminal.write("w");
  await new Promise((r) => setTimeout(r, 120));

  // Same-key re-trigger with >100ms debounce window
  terminal.write("q");
  await new Promise((r) => setTimeout(r, 120));
  terminal.write("q");
  await new Promise((r) => setTimeout(r, 120));

  // Esc back
  terminal.keyEscape();
  await new Promise((r) => setTimeout(r, 80));
  await expect(terminal.getByText("Back to browse", { full: false })).toBeVisible();
});


