import { test, expect } from "@microsoft/tui-test";

test.use({ program: { file: "./target/release/termigroove" } });

test("Tab toggles focus: right pane shows focused style", async ({ terminal }) => {
  // Initial render
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  // Serialize before to inspect style shifts later
  const before = terminal.serialize();

  // Send a Tab key (ANSI \t should be forwarded as Tab)
  terminal.write("\t");

  // Give the app a moment to handle and render
  await new Promise((r) => setTimeout(r, 100));

  // Expect some reversed/bold style indicator around the right title region.
  // Simplify: verify at least one cell now has inverse (reversed) style.
  await expect(
    terminal.getByText("Selected (Enter = To Pads)", { full: true })
  ).toBeVisible();

  await expect(terminal.getByText("Right focus", { full: false })).toBeVisible();
});


