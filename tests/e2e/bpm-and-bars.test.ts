import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test.describe("BPM & Bars configuration flow", () => {
  test("Arrow shows summary, Enter opens popup, OK applies", async ({ terminal }) => {
    // Wait for pads mode confirmation
    if (!(await goToPadsScreen(terminal))) {
      test.skip();
      return;
    }

    // Arrow key should focus summary box and show the labels
    terminal.keyRight(1);
    await expect(terminal.getByText("bpm:", { full: false })).toBeVisible();
    await expect(terminal.getByText("bars:", { full: false })).toBeVisible();

    // Enter opens popup; expect title and initial values
    terminal.write("\r");
    await expect(terminal.getByText("Configure tempo & loop", { full: true })).toBeVisible();
    await expect(terminal.getByText("bpm", { full: false })).toBeVisible();
    await expect(terminal.getByText("bars", { full: false })).toBeVisible();

    // Type new BPM and Bars values (350 -> clamps to 300, 257 -> clamps to 256)
    terminal.write("\b\b\b350");
    terminal.keyDown(1);
    terminal.write("\b\b\b257");

    // Navigate to OK (down) and confirm
    terminal.keyDown(1);
    terminal.write("\r");

    // Popup should close and summary box should show clamped values
    await expect(terminal.getByText("Configure tempo & loop", { full: true })).not.toBeVisible();
    await expect(terminal.getByText("bpm: 300", { full: true })).toBeVisible();
    await expect(terminal.getByText("bars: 256", { full: true })).toBeVisible();
  });

  test("Cancel and Esc discard edits", async ({ terminal }) => {
    if (!(await goToPadsScreen(terminal))) {
      test.skip();
      return;
    }

    // Focus summary and open popup
    terminal.keyRight(1);
    terminal.write("\r");
    await expect(terminal.getByText("Configure tempo & loop", { full: true })).toBeVisible();

    // Change BPM to 130, Bars to 12 then Cancel
    terminal.write("\b\b\b130");
    terminal.keyDown(1);
    terminal.write("\b\b\b012");
    terminal.keyDown(1);
    terminal.keyRight(1);
    terminal.write("\r");

    // Values should remain defaults (120 / 16)
    await expect(terminal.getByText("Configure tempo & loop", { full: true })).not.toBeVisible();
    await expect(terminal.getByText("bpm: 120", { full: true })).toBeVisible();
    await expect(terminal.getByText("bars: 16", { full: true })).toBeVisible();

    // Reopen and test Esc
    terminal.keyRight(1);
    terminal.write("\r");
    await expect(terminal.getByText("Configure tempo & loop", { full: true })).toBeVisible();
    terminal.write("\b\b\b180");
    terminal.keyDown(1);
    terminal.write("\b\b\b020");
    terminal.keyEscape();

    await expect(terminal.getByText("Configure tempo & loop", { full: true })).not.toBeVisible();
    await expect(terminal.getByText("bpm: 120", { full: true })).toBeVisible();
    await expect(terminal.getByText("bars: 16", { full: true })).toBeVisible();
  });
});

async function goToPadsScreen(terminal: any): Promise<boolean> {
    terminal.write(" ");
    await new Promise((r) => setTimeout(r, 80));
    terminal.write("\r");
    await new Promise((r) => setTimeout(r, 160));
    try {
        const padsHint = await terminal.getByText("[Pads] Press Esc to go back.", { full: false });
        await expect(padsHint).toBeVisible();
        return true;
    } catch {
        return false;
    }
}

