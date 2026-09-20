/**
 * Bundles the whole service into one file for Tencent Cloud SCF.
 *
 * A single file means the code can be pasted straight into the SCF console's
 * online editor, which sidesteps the Zip problem on Windows: PowerShell's
 * Compress-Archive cannot set the Unix 755 bit that `scf_bootstrap` needs.
 */

import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const outDir = join(root, "dist", "scf");
const outFile = join(outDir, "index.mjs");

await mkdir(outDir, { recursive: true });

const result = await build({
  entryPoints: [join(root, "src", "node.js")],
  outfile: outFile,
  bundle: true,
  platform: "node",
  format: "esm",
  target: "node18",
  legalComments: "none",
  logLevel: "warning",
  metafile: true,
});

await writeFile(join(outDir, "package.json"), `{\n  "type": "module"\n}\n`, "utf8");

// SCF 只认 `scf_bootstrap` 这个名字，并且要求内容以 LF 换行、文件有可执行权限。
// 在控制台里把同样的内容填进「高级配置 > 启动命令」也一样有效，还能绕开权限问题。
await writeFile(
  join(outDir, "scf_bootstrap"),
  [
    "#!/bin/bash",
    "export PORT=9000",
    "export STATE_FILE=/tmp/glossy-cloud-quota.json",
    "exec /var/lang/node18/bin/node index.mjs",
    "",
  ].join("\n"),
  "utf8",
);

const bytes = Object.values(result.metafile.outputs)[0]?.bytes ?? 0;
console.log(`dist/scf/index.mjs        ${(bytes / 1024).toFixed(1)} KB`);
console.log("dist/scf/scf_bootstrap    启动命令（LF 换行，755 权限）");
console.log("dist/scf/package.json     type=module");
