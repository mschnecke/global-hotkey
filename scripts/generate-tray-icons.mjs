/**
 * Generate tray icons from artwork
 *
 * Creates:
 * - tray-icon.png (macOS template @1x - 22x22, white with alpha)
 * - tray-icon@2x.png (macOS template @2x - 44x44, white with alpha)
 * - tray-icon-light.png (Windows light mode - dark icon, 32x32)
 * - tray-icon-dark.png (Windows dark mode - light icon, 32x32)
 */

import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const artworkPath = path.join(rootDir, 'artwork', 'Copilot_20251215_162130.png');
const outputDir = path.join(rootDir, 'src-tauri', 'icons');

async function generateTrayIcons() {
  console.warn('Loading artwork from:', artworkPath);

  // Load the source image
  const sourceImage = sharp(artworkPath);
  const metadata = await sourceImage.metadata();
  console.warn('Source image size:', metadata.width, 'x', metadata.height);

  // The source has a dark background - we need to extract the white content
  // First, let's get the raw pixel data
  const { data, info } = await sourceImage
    .raw()
    .toBuffer({ resolveWithObject: true });

  console.warn('Processing image...');

  // Create a version with transparent background (extract white/light content)
  // The icon content is white/light colored on dark background
  const transparentBuffer = Buffer.alloc(info.width * info.height * 4);

  for (let i = 0; i < info.width * info.height; i++) {
    const srcIdx = i * info.channels;
    const dstIdx = i * 4;

    const r = data[srcIdx];
    const g = data[srcIdx + 1];
    const b = data[srcIdx + 2];

    // Calculate luminance - the icon content is light/white
    const luminance = 0.299 * r + 0.587 * g + 0.114 * b;

    // The background is dark (~60), the icon is light (~200+)
    // Use luminance to determine alpha (icon visibility)
    if (luminance > 100) {
      // This is icon content - make it white with alpha based on luminance
      transparentBuffer[dstIdx] = 255; // R
      transparentBuffer[dstIdx + 1] = 255; // G
      transparentBuffer[dstIdx + 2] = 255; // B
      // Alpha: scale from luminance (higher luminance = more opaque)
      transparentBuffer[dstIdx + 3] = Math.min(255, Math.round((luminance - 100) * 1.6));
    } else {
      // Background - make transparent
      transparentBuffer[dstIdx] = 0;
      transparentBuffer[dstIdx + 1] = 0;
      transparentBuffer[dstIdx + 2] = 0;
      transparentBuffer[dstIdx + 3] = 0;
    }
  }

  // Create base transparent image
  const transparentImage = sharp(transparentBuffer, {
    raw: {
      width: info.width,
      height: info.height,
      channels: 4,
    },
  });

  // macOS template icon @1x (22x22) - white with alpha
  console.warn('Generating tray-icon.png (macOS @1x, 22x22)...');
  await transparentImage
    .clone()
    .resize(22, 22, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(path.join(outputDir, 'tray-icon.png'));

  // macOS template icon @2x (44x44) - white with alpha
  console.warn('Generating tray-icon@2x.png (macOS @2x, 44x44)...');
  await transparentImage
    .clone()
    .resize(44, 44, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(path.join(outputDir, 'tray-icon@2x.png'));

  // Windows dark mode icon (32x32) - white/light icon for dark backgrounds
  console.warn('Generating tray-icon-dark.png (Windows dark mode, 32x32)...');
  await transparentImage
    .clone()
    .resize(32, 32, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(path.join(outputDir, 'tray-icon-dark.png'));

  // For Windows light mode, we need a dark icon
  // Create a dark version by inverting the colors but keeping alpha
  const darkBuffer = Buffer.alloc(info.width * info.height * 4);

  for (let i = 0; i < info.width * info.height; i++) {
    const srcIdx = i * 4;
    const dstIdx = i * 4;

    const alpha = transparentBuffer[srcIdx + 3];

    if (alpha > 0) {
      // Make it dark gray instead of white
      darkBuffer[dstIdx] = 64; // R - dark gray
      darkBuffer[dstIdx + 1] = 64; // G
      darkBuffer[dstIdx + 2] = 64; // B
      darkBuffer[dstIdx + 3] = alpha; // Keep same alpha
    } else {
      darkBuffer[dstIdx] = 0;
      darkBuffer[dstIdx + 1] = 0;
      darkBuffer[dstIdx + 2] = 0;
      darkBuffer[dstIdx + 3] = 0;
    }
  }

  // Windows light mode icon (32x32) - dark icon for light backgrounds
  console.warn('Generating tray-icon-light.png (Windows light mode, 32x32)...');
  await sharp(darkBuffer, {
    raw: {
      width: info.width,
      height: info.height,
      channels: 4,
    },
  })
    .resize(32, 32, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(path.join(outputDir, 'tray-icon-light.png'));

  console.warn('\nAll tray icons generated successfully!');
  console.warn('Output directory:', outputDir);
}

generateTrayIcons().catch(console.error);
