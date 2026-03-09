/**
 * Escape a CSV field value
 * - If the field contains comma, quote, or newline, wrap it in quotes
 * - Escape any quotes by doubling them
 */
export function escapeCSVField(value: string): string {
  if (value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r')) {
    return '"' + value.replace(/"/g, '""') + '"';
  }
  return value;
}

/**
 * Unescape a CSV field value
 * - Remove surrounding quotes if present
 * - Unescape doubled quotes
 */
export function unescapeCSVField(value: string): string {
  if (value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1).replace(/""/g, '"');
  }
  return value;
}

/**
 * Parse a CSV line handling quoted fields
 */
export function parseCSVLine(line: string): string[] {
  const fields: string[] = [];
  let currentField = '';
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];

    if (inQuotes) {
      if (char === '"') {
        // Check if this is an escaped quote
        if (i + 1 < line.length && line[i + 1] === '"') {
          currentField += '"';
          i++; // Skip next quote
        } else {
          // End of quoted field
          inQuotes = false;
        }
      } else {
        currentField += char;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
      } else if (char === ',') {
        fields.push(unescapeCSVField(currentField));
        currentField = '';
      } else {
        currentField += char;
      }
    }
  }

  // Add the last field
  fields.push(unescapeCSVField(currentField));

  return fields;
}

/**
 * Clean a field value by removing carriage returns and trimming whitespace
 */
export function cleanFieldValue(value: string): string {
  return value.replace(/\r/g, '').trim();
}
