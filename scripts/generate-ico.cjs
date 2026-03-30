const sharp = require('sharp');
const fs = require('fs');
const path = require('path');
const { default: pngToIco } = require('png-to-ico');

async function generateIco() {
  const svgPath = path.join(__dirname, '..', 'src-tauri', 'icons', 'icon.svg');
  const icoPath = path.join(__dirname, '..', 'src-tauri', 'icons', 'icon.ico');

  // 读取 SVG
  const svgBuffer = fs.readFileSync(svgPath);

  // 生成不同尺寸的 PNG
  const sizes = [16, 24, 32, 48, 64, 128, 256];
  const pngBuffers = [];

  for (const size of sizes) {
    console.log(`Generating ${size}x${size}...`);
    const pngBuffer = await sharp(svgBuffer)
      .resize(size, size, { fit: 'contain', background: { r: 59, g: 130, b: 246, alpha: 1 } })
      .png()
      .toBuffer();
    pngBuffers.push(pngBuffer);
  }

  // 转换为 ICO
  const icoBuffer = await pngToIco(pngBuffers);
  fs.writeFileSync(icoPath, icoBuffer);

  console.log(`ICO file generated at: ${icoPath}`);
  console.log(`Included sizes: ${sizes.join(', ')}`);
}

generateIco().catch(console.error);
