import fs from "node:fs";
import path from "node:path";

// Write a small 32-bit Windows ICO without requiring an image package.
const size = 64;
const xorMask = Buffer.alloc(size * size * 4);
const andMask = Buffer.alloc(size * Math.ceil(size / 32) * 4);

function insidePolygon(x, y, points) {
  let inside = false;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) {
    const [xi, yi] = points[i];
    const [xj, yj] = points[j];
    const crosses = yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;
    if (crosses) inside = !inside;
  }
  return inside;
}

for (let fileY = 0; fileY < size; fileY += 1) {
  const y = size - 1 - fileY;
  for (let x = 0; x < size; x += 1) {
    const dx = x - 30;
    const dy = y - 29;
    const radius = Math.hypot(dx, dy);
    let color = [23, 50, 79, 255];
    if (radius >= 18 && radius <= 22) color = [114, 225, 195, 255];
    if (radius <= 4) color = [249, 250, 251, 255];
    if (insidePolygon(x, y, [[36, 35], [51, 58], [45, 57], [41, 51], [37, 57]])) {
      color = [249, 250, 251, 255];
    }
    const offset = (fileY * size + x) * 4;
    xorMask[offset] = color[2];
    xorMask[offset + 1] = color[1];
    xorMask[offset + 2] = color[0];
    xorMask[offset + 3] = color[3];
  }
}

const bitmapHeader = Buffer.alloc(40);
bitmapHeader.writeUInt32LE(40, 0);
bitmapHeader.writeInt32LE(size, 4);
bitmapHeader.writeInt32LE(size * 2, 8);
bitmapHeader.writeUInt16LE(1, 12);
bitmapHeader.writeUInt16LE(32, 14);
bitmapHeader.writeUInt32LE(0, 16);
bitmapHeader.writeUInt32LE(xorMask.length, 20);

const imageData = Buffer.concat([bitmapHeader, xorMask, andMask]);
const directory = Buffer.alloc(22);
directory.writeUInt16LE(0, 0);
directory.writeUInt16LE(1, 2);
directory.writeUInt16LE(1, 4);
directory.writeUInt8(size, 6);
directory.writeUInt8(size, 7);
directory.writeUInt16LE(1, 10);
directory.writeUInt16LE(32, 12);
directory.writeUInt32LE(imageData.length, 14);
directory.writeUInt32LE(directory.length, 18);

const outputPath = path.join("src-tauri", "icons", "icon.ico");
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, Buffer.concat([directory, imageData]));
