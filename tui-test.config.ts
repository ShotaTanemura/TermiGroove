import { defineConfig } from "@microsoft/tui-test";

export default defineConfig({
  retries: 2,
  trace: true,
  testMatch: "tests/e2e/**/*.@(spec|test).?(c|m)[jt]s?(x)",
  use: {
    program: {
      file: "./target/release/termigroove",
      args: [],
    },
    env: { 
      RUST_BACKTRACE: "full", 
      LC_ALL: "C", 
      COLUMNS: "120", 
      LINES: "40" 
    },
    columns: 120,
    rows: 40
  }
});
