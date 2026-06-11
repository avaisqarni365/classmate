import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const cargoDir = join(homedir(), ".cargo", "bin");
const cargoName = process.platform === "win32" ? "cargo.exe" : "cargo";
const cargoPath = join(cargoDir, cargoName);

if (!existsSync(cargoPath)) {
  console.error("");
  console.error("Rust/Cargo was not found.");
  console.error("");
  console.error("Install Rust, then restart Cursor:");
  console.error("  winget install Rustlang.Rustup");
  console.error("  or visit https://rustup.rs");
  console.error("");
  process.exit(1);
}

const pathKey = process.platform === "win32" ? "Path" : "PATH";
const separator = process.platform === "win32" ? ";" : ":";
const env = {
  ...process.env,
  [pathKey]: `${cargoDir}${separator}${process.env[pathKey] ?? ""}`,
};

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error("Usage: node scripts/run-tauri.mjs <dev|build|...>");
  process.exit(1);
}

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const tauriBin = join(
  projectRoot,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tauri.cmd" : "tauri",
);

const result = spawnSync(tauriBin, args, {
  stdio: "inherit",
  env,
  shell: true,
  cwd: projectRoot,
});

process.exit(result.status ?? 1);
