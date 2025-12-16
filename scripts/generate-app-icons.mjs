/**
 * Generate application icons from SVG source
 *
 * Creates:
 * - icon.png (512x512)
 * - 32x32.png
 * - 128x128.png
 * - 128x128@2x.png (256x256)
 * - icon.ico (Windows multi-size)
 * - icon.icns (macOS multi-size)
 * - Windows Store assets (Square*.png, StoreLogo.png)
 */

import sharp from 'sharp';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import fs from 'fs';
import pngToIco from 'png-to-ico';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');
const svgPath = path.join(rootDir, 'app-icon.svg');
const outputDir = path.join(rootDir, 'src-tauri', 'icons');

// Icon sizes for various purposes
const pngSizes = [
  { size: 32, name: '32x32.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' },
  { size: 512, name: 'icon.png' },
];

// Windows Store icons
const storeIcons = [
  { size: 30, name: 'Square30x30Logo.png' },
  { size: 44, name: 'Square44x44Logo.png' },
  { size: 71, name: 'Square71x71Logo.png' },
  { size: 89, name: 'Square89x89Logo.png' },
  { size: 107, name: 'Square107x107Logo.png' },
  { size: 142, name: 'Square142x142Logo.png' },
  { size: 150, name: 'Square150x150Logo.png' },
  { size: 284, name: 'Square284x284Logo.png' },
  { size: 310, name: 'Square310x310Logo.png' },
  { size: 50, name: 'StoreLogo.png' },
];

// ICO sizes (Windows requires multiple sizes)
const icoSizes = [16, 24, 32, 48, 64, 128, 256];

// ICNS sizes (macOS)
const icnsSizes = [16, 32, 64, 128, 256, 512, 1024];

async function generatePngIcons() {
  console.warn('Generating PNG icons...');

  for (const { size, name } of pngSizes) {
    console.warn(`  ${name} (${size}x${size})`);
    await sharp(svgPath).resize(size, size).png().toFile(path.join(outputDir, name));
  }
}

async function generateStoreIcons() {
  console.warn('Generating Windows Store icons...');

  for (const { size, name } of storeIcons) {
    console.warn(`  ${name} (${size}x${size})`);
    await sharp(svgPath).resize(size, size).png().toFile(path.join(outputDir, name));
  }
}

async function generateIco() {
  console.warn('Generating icon.ico...');

  // Generate temporary PNGs for ICO
  const tempDir = path.join(outputDir, '.ico-temp');
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true });
  }

  const pngPaths = [];
  for (const size of icoSizes) {
    const pngPath = path.join(tempDir, `${size}.png`);
    await sharp(svgPath).resize(size, size).png().toFile(pngPath);
    pngPaths.push(pngPath);
  }

  // Create ICO
  const icoBuffer = await pngToIco(pngPaths);
  fs.writeFileSync(path.join(outputDir, 'icon.ico'), icoBuffer);

  // Clean up temp files
  for (const pngPath of pngPaths) {
    fs.unlinkSync(pngPath);
  }
  fs.rmdirSync(tempDir);

  console.warn('  icon.ico created');
}

async function generateIcns() {
  console.warn('Generating icon.icns...');

  // Create iconset directory (required by iconutil)
  const iconsetDir = path.join(outputDir, 'icon.iconset');
  if (!fs.existsSync(iconsetDir)) {
    fs.mkdirSync(iconsetDir, { recursive: true });
  }

  // Generate all required sizes for ICNS
  // macOS expects specific naming: icon_NxN.png and icon_NxN@2x.png
  const icnsFiles = [
    { size: 16, name: 'icon_16x16.png' },
    { size: 32, name: 'icon_16x16@2x.png' },
    { size: 32, name: 'icon_32x32.png' },
    { size: 64, name: 'icon_32x32@2x.png' },
    { size: 128, name: 'icon_128x128.png' },
    { size: 256, name: 'icon_128x128@2x.png' },
    { size: 256, name: 'icon_256x256.png' },
    { size: 512, name: 'icon_256x256@2x.png' },
    { size: 512, name: 'icon_512x512.png' },
    { size: 1024, name: 'icon_512x512@2x.png' },
  ];

  for (const { size, name } of icnsFiles) {
    await sharp(svgPath).resize(size, size).png().toFile(path.join(iconsetDir, name));
  }

  // Use iconutil to create ICNS (macOS only)
  try {
    execSync(`iconutil -c icns -o "${path.join(outputDir, 'icon.icns')}" "${iconsetDir}"`, {
      stdio: 'pipe',
    });
    console.warn('  icon.icns created');
  } catch (error) {
    console.warn('  Warning: iconutil failed (only works on macOS)');
    console.warn('  You may need to generate icon.icns manually on a Mac');
  }

  // Clean up iconset directory
  const files = fs.readdirSync(iconsetDir);
  for (const file of files) {
    fs.unlinkSync(path.join(iconsetDir, file));
  }
  fs.rmdirSync(iconsetDir);
}

async function main() {
  console.warn('Loading SVG from:', svgPath);
  console.warn('Output directory:', outputDir);
  console.warn('');

  await generatePngIcons();
  await generateStoreIcons();
  await generateIco();
  await generateIcns();

  console.warn('\nAll app icons generated successfully!');
}

main().catch(console.error);
