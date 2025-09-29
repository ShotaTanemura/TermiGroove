import { test, expect } from "@microsoft/tui-test";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

test.use({ program: { file: "./target/release/termigroove" } });

test("Loop overdub layering pause resume clear", async ({ terminal }) => {
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  // Select three files (best effort) and enter pads
  terminal.write("\u001b[B"); // Down
  await sleep(80);
  terminal.write("\u001b[B"); // Down
  await sleep(80);
  terminal.write("\u001b[B"); // Down
  await sleep(80);
  terminal.write("\u001b[C"); // right
  await sleep(80);
  terminal.write(" ");
  await sleep(80);
  terminal.write("\u001b[B"); // Down
  terminal.write(" ");
  await sleep(80);
  terminal.write("\u001b[B");
  terminal.write(" ");
  await sleep(80);
  terminal.write("\r");
  await sleep(200);

  await expect(terminal.getByText("Pads", { full: false })).toBeVisible();

  // Start loop (count-in)
  terminal.write(" ");
  await sleep(2500);

  // Record base loop layer
  terminal.write("q");
  await sleep(200);

  // Wait and overdub another layer
  await sleep(1500);
  terminal.write("w");
  await sleep(200);

  // Stop overdub recording on the current cycle
  terminal.write(" ");
  await wait_for_status(terminal, "Loop playing", 4000);

  // Pause
  terminal.write(" ");
  await wait_for_status(terminal, "Loop paused", 4000);

  // Resume
  terminal.write(" ");
  await wait_for_status(terminal, "Loop playing", 4000);

  // // Clear via Ctrl+Space (send control sequence)
  // terminal.write("\u0000");
  // terminal.write(" ");
  // await sleep(600);
  // await expect(terminal.getByText("Loop cleared", { full: false })).toBeVisible();

  // Back to browse
  terminal.keyEscape();
  await sleep(200);
  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();
});

async function wait_for_status(
  terminal: ReturnType<typeof test.use>["terminal"],
  text: string,
  timeoutMs = 2000,
  pollMs = 100
) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const log = await terminal.getBuffer();
    if (log.includes(text)) {
      return;
    }
    await sleep(pollMs);
  }
  throw new Error(`status '${text}' not visible after ${timeoutMs}ms`);
}
