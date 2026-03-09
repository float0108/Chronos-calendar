/**
 * Detect if a buffer contains UTF-8 BOM
 */
export function hasBOM(buffer: Uint8Array): boolean {
  return buffer.length >= 3 && buffer[0] === 0xEF && buffer[1] === 0xBB && buffer[2] === 0xBF;
}

/**
 * Try to decode buffer as UTF-8, fallback to GBK or Windows-1252
 */
export function decodeCSVBuffer(buffer: Uint8Array): string {
  // Try UTF-8 first
  const utf8Decoder = new TextDecoder('utf-8', { fatal: false });

  // If has BOM, skip it
  if (hasBOM(buffer)) {
    return utf8Decoder.decode(buffer.slice(3));
  }

  // Try UTF-8
  const utf8Text = utf8Decoder.decode(buffer);

  // Check for replacement characters that indicate decoding issues
  const replacementCount = (utf8Text.match(/\uFFFD/g) || []).length;
  const totalChars = utf8Text.length;

  // If less than 5% are replacement characters, it's probably UTF-8
  if (replacementCount / totalChars < 0.05) {
    return utf8Text;
  }

  // Fallback to GBK for Chinese Windows systems
  try {
    const gbkDecoder = new TextDecoder('gbk');
    const gbkText = gbkDecoder.decode(buffer);

    // Check if GBK decoding looks reasonable (contains Chinese characters)
    const chineseCharCount = (gbkText.match(/[\u4e00-\u9fa5]/g) || []).length;
    if (chineseCharCount > 0) {
      return gbkText;
    }
  } catch (error) {
    console.warn('GBK decoding failed:', error);
  }

  // Fallback to Windows-1252 (ANSI)
  try {
    const ansiDecoder = new TextDecoder('windows-1252');
    return ansiDecoder.decode(buffer);
  } catch (error) {
    console.error('Failed to decode file:', error);
    throw new Error('无法识别文件编码，请使用 UTF-8 或 GBK 编码');
  }
}
