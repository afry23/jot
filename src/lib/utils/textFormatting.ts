// Text formatting utility functions

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

// Format markdown text to HTML for preview
export function formatMarkdown(text: string): string {
  if (!text) return '';
  
  let formatted = text;
  
  // Simple sanitization (prevent script injection)
  formatted = formatted.replace(/</g, '&lt;').replace(/>/g, '&gt;');
  
  // Format headers (# Header)
  formatted = formatted.replace(/^# (.+)$/gm, '<h1>$1</h1>');
  formatted = formatted.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  formatted = formatted.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  
  // Format bold (**bold**)
  formatted = formatted.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  
  // Format italic (*italic*)
  formatted = formatted.replace(/\*(.+?)\*/g, '<em>$1</em>');
  
  // Format code blocks (```code```)
  formatted = formatted.replace(/```([\s\S]+?)```/g, '<pre><code>$1</code></pre>');
  
  // Format inline code (`code`)
  formatted = formatted.replace(/`([^`]+)`/g, '<code>$1</code>');
  
  // Format blockquotes (> quote)
  formatted = formatted.replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>');
  
  // Format unordered lists (- item)
  formatted = formatted.replace(/^- (.+)$/gm, '<ul><li>$1</li></ul>');
  
  // Fix nested lists (combine adjacent ul tags)
  formatted = formatted.replace(/<\/ul>\s*<ul>/g, '');
  
  // Format ordered lists (1. item)
  formatted = formatted.replace(/^\d+\. (.+)$/gm, '<ol><li>$1</li></ol>');
  
  // Fix nested ordered lists
  formatted = formatted.replace(/<\/ol>\s*<ol>/g, '');
  
  // Format horizontal rules (---)
  formatted = formatted.replace(/^---+$/gm, '<hr>');
  
  // Format links ([text](url))
  formatted = formatted.replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>');
  
  // Convert line breaks to <br> tags, avoiding adding them inside code blocks
  formatted = formatted.replace(/(?!<pre>|<\/pre>|<code>|<\/code>)\n(?!<pre>|<\/pre>|<code>|<\/code>)/g, '<br>');
  
  // Wrap plain text in paragraphs (only text not already in HTML tags)
  // This is a bit complex and may introduce nesting issues
  /*
  const paragraphs = formatted.split(/(?:<\/?[a-z]+(?:\s+[^>]*)?>)+/g);
  for (let i = 0; i < paragraphs.length; i++) {
    if (paragraphs[i].trim() && !paragraphs[i].match(/^<\//)) {
      formatted = formatted.replace(paragraphs[i], `<p>${paragraphs[i]}</p>`);
    }
  }
  */
  
  return formatted;
}