/**
 * Generate tray icons from SVG source
 *
 * Source SVG uses BLACK color for macOS template compatibility.
 * macOS template images use alpha channel as the mask, with black pixels.
 *
 * Creates:
 * - tray-icon.png (macOS template @1x - 22x22, black with alpha)
 * - tray-icon@2x.png (macOS template @2x - 44x44, black with alpha)
 * - tray-icon-light.png (Windows light mode - dark icon, 32x32)
 * - tray-icon-dark.png (Windows dark mode - white icon, 32x32)
 */

import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const svgPath = path.join(rootDir, 'artwork', 'tray-icon-source.svg');
const outputDir = path.join(rootDir, 'src-tauri', 'icons');

async function generateTrayIcons() {
  console.warn('Loading SVG from:', svgPath);

  // macOS template icon @1x (22x22) - black with alpha (macOS will tint)
  console.warn('Generating tray-icon.png (macOS @1x, 22x22)...');
  await sharp(svgPath).resize(22, 22).png().toFile(path.join(outputDir, 'tray-icon.png'));

  // macOS template icon @2x (44x44) - black with alpha
  console.warn('Generating tray-icon@2x.png (macOS @2x, 44x44)...');
  await sharp(svgPath).resize(44, 44).png().toFile(path.join(outputDir, 'tray-icon@2x.png'));

  // Windows light mode icon (32x32) - dark icon (can use black directly)
  console.warn('Generating tray-icon-light.png (Windows light mode, 32x32)...');
  await sharp(svgPath).resize(32, 32).png().toFile(path.join(outputDir, 'tray-icon-light.png'));

  // For Windows dark mode, we need a white icon
  // Render the SVG then transform black to white
  console.warn('Generating tray-icon-dark.png (Windows dark mode, 32x32)...');
  const blackIcon = await sharp(svgPath).resize(32, 32).raw().toBuffer({ resolveWithObject: true });

  const { data, info } = blackIcon;
  const whiteBuffer = Buffer.alloc(info.width * info.height * 4);

  for (let i = 0; i < info.width * info.height; i++) {
    const srcIdx = i * info.channels;
    const dstIdx = i * 4;

    const r = data[srcIdx];
    const g = data[srcIdx + 1];
    const b = data[srcIdx + 2];
    const a = info.channels === 4 ? data[srcIdx + 3] : 255;

    // If pixel has alpha (is part of the icon), make it white
    if (a > 0) {
      whiteBuffer[dstIdx] = 255; // R - white
      whiteBuffer[dstIdx + 1] = 255; // G
      whiteBuffer[dstIdx + 2] = 255; // B
      whiteBuffer[dstIdx + 3] = a; // Keep alpha
    } else {
      // Transparent
      whiteBuffer[dstIdx] = 0;
      whiteBuffer[dstIdx + 1] = 0;
      whiteBuffer[dstIdx + 2] = 0;
      whiteBuffer[dstIdx + 3] = 0;
    }
  }

  await sharp(whiteBuffer, {
    raw: {
      width: info.width,
      height: info.height,
      channels: 4,
    },
  })
    .png()
    .toFile(path.join(outputDir, 'tray-icon-dark.png'));

  console.warn('\nAll tray icons generated successfully!');
  console.warn('Output directory:', outputDir);
}

generateTrayIcons().catch(console.error);
