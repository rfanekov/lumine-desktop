import sharp from "sharp";
import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 1024;
const LOGO = 872;
const PAD = (SIZE - LOGO) / 2;
const TRANSPARENT = { r: 0, g: 0, b: 0, alpha: 0 };

const logoPath = resolve(fileURLToPath(new URL("../public/logo.webp", import.meta.url)));
const output = resolve(fileURLToPath(new URL("../assets/icon.png", import.meta.url)));
mkdirSync(dirname(output), { recursive: true });

await sharp(logoPath)
  .resize(LOGO, LOGO, { fit: "contain", background: TRANSPARENT })
  .extend({ top: PAD, bottom: PAD, left: PAD, right: PAD, background: TRANSPARENT })
  .png()
  .toFile(output);

console.log(`Generated ${output}`);
