// Ajoute à notes.md la somme SHA-256 de chaque installateur (.exe) produit par la construction.
// Appelé par .github/workflows/release.yml ; ARTIFACTS = liste JSON des fichiers (sortie « artifactPaths » de tauri-action).
import { appendFileSync, readFileSync } from "node:fs";
import { basename } from "node:path";
import { createHash } from "node:crypto";

const exes = JSON.parse(process.env.ARTIFACTS || "[]").filter((p) => p.toLowerCase().endsWith(".exe"));
if (!exes.length) process.exit(0);

let out = "\n\n### Somme SHA-256 / SHA-256 checksum\n";
for (const p of exes) {
  const sum = createHash("sha256").update(readFileSync(p)).digest("hex");
  out += `\n\`${basename(p)}\`\n\`\`\`\n${sum}\n\`\`\`\n`;
}
out += "\nPowerShell : `Get-FileHash <fichier>.exe -Algorithm SHA256`\n";
appendFileSync(process.argv[2] || "notes.md", out);
