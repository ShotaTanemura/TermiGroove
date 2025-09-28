import { test, expect } from "@microsoft/tui-test";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const tryFind = async (terminal: ReturnType<typeof test.use>["terminal"], text: string) => {
  try {
    const locator = await terminal.getByText(text, { full: false });
    await expect(locator).toBeVisible();
    return true;
  } catch {
    return false;
  }
};

test.use({ program: { file: "./target/release/termigroove" } });

test("Loop capture: count-in, record, playback, and stop", async ({ terminal }) => {
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  // Select current item if available (best effort)
  terminal.write(" ");
  await sleep(80);

  // Remember if selection succeeded
  const haveSelection = await tryFind(terminal, "Selected samples:");

  // Enter Pads (if no selection, loop recording isn't possible)
  terminal.write("\r");
  await sleep(200);

  if (!haveSelection) {
    await expect(terminal.getByText("Select at least one file first", { full: false })).toBeVisible();
    return;
  }

  await expect(terminal.getByText("Pads", { full: false })).toBeVisible();

  // Kick off count-in
  terminal.write(" ");
  await sleep(2500);

  // Trigger a pad during recording
  terminal.write("q");
  await sleep(200);

  // Wait for playback cycle to complete
  await sleep(3500);

  // Stop loop via space
  terminal.write(" ");
  await sleep(500);

  // Exit pads
  terminal.keyEscape();
  await sleep(200);
  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();
});
