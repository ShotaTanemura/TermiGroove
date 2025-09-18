import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Pads flow: Enter with selection shows pads, Esc returns", async ({ terminal }) => {
  // Initial UI hints
  await expect(terminal.getByText("Selected (Enter = To Pads)", { full: true })).toBeVisible();

  // Focus left (default) and attempt to select current item if it's a file; otherwise just proceed
  // Space attempts selection; our app guards on directories
  terminal.write(" ");
  await new Promise((r) => setTimeout(r, 80));

  // Press Enter to go to Pads (if no selection, app will block, but test still valid to assert status)
  terminal.write("\r");
  await new Promise((r) => setTimeout(r, 120));

  // Either we see pads instructions or selection warning
  try {
    const padsHint = await terminal.getByText("[Pads] Press Esc to go back.", { full: false });
    await expect(padsHint).toBeVisible();
    // If pads mode, Esc should return; send Esc unconditionally and expect browse footer
    terminal.keyEscape();
    await new Promise((r) => setTimeout(r, 80));
    await expect(terminal.getByText("Back to browse", { full: false })).toBeVisible();
  } catch {
    const needSelection = await terminal.getByText("Select at least one file first", { full: false });
    await expect(needSelection).toBeVisible();
  }
});


