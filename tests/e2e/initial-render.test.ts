import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("initial render shows header, panes and footer", async ({ terminal }) => {
  await expect(
    terminal.getByText("Load your samples...", { full: true })
  ).toBeVisible();

  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  await expect(terminal.getByText("Ready", { full: false })).toBeVisible();
});



