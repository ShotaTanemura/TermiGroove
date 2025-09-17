import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Resize keeps UI stable and focus visuals intact", async ({ terminal }) => {
  // Verify initial right title visible
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  // Switch focus to right and check footer status
  terminal.write("\t");
  await new Promise((r) => setTimeout(r, 50));
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();

  // Resize the terminal (smaller then larger)
  terminal.resize(100, 30);
  await new Promise((r) => setTimeout(r, 50));
  terminal.resize(120, 40);
  await new Promise((r) => setTimeout(r, 50));

  // UI remains stable: right title visible; focus status preserved
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();
  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();
});


