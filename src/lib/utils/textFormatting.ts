export function countWords(text: string): number {
  return text.trim().split(/\s+/).filter(word => word.length > 0).length;
}

// Count characters in text
export function countCharacters(text: string): number {
  return text.length;
}

// Count lines in text
export function countLines(text: string): number {
  return text.split('\n').length;
}