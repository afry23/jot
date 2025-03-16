// Basic rich text formatting functions

// Format selected text as bold
export function formatBold() {
    document.execCommand('bold', false);
  }
  
  // Format selected text as italic
  export function formatItalic() {
    document.execCommand('italic', false);
  }
  
  // Format selected text as a list
  export function formatList() {
    document.execCommand('insertUnorderedList', false);
  }
  
  // Format selected text as a heading
  export function formatHeading(level: 1 | 2 | 3) {
    document.execCommand('formatBlock', false, `h${level}`);
  }
  
  // Clear formatting from selected text
  export function clearFormatting() {
    document.execCommand('removeFormat', false);
  }
  
  // Get plain text from HTML content
  export function getPlainText(html: string): string {
    const tempElement = document.createElement('div');
    tempElement.innerHTML = html;
    return tempElement.textContent || '';
  }
  
  // Count words in text
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