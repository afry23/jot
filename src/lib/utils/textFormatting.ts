import DOMPurify from "dompurify";
import { marked } from "marked";
import { getHeadingColorWithOpacity } from "./uiColors";

// Count words in text
export function countWords(text: string): number {
  if (!text || text.trim() === "") return 0;

  // Strip markdown syntax first
  const strippedText = text
    // Remove headings
    .replace(/^#+\s+/gm, "")
    // Remove bold and italic markers
    .replace(/\*\*|\*/g, "")
    // Remove list markers
    .replace(/^[\s-]*[-*+]\s+/gm, "")
    // Remove blockquotes
    .replace(/^>\s+/gm, "")
    // Remove horizontal rules
    .replace(/^-{3,}$/gm, "")
    // Remove link syntax while preserving link text
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    // Remove inline code markers while preserving code content
    .replace(/`([^`]*)`/g, "$1")
    // Remove code block markers while preserving code content
    .replace(/```(?:\w+)?\s*([\s\S]*?)```/g, "$1")
    // Remove checkbox syntax from task lists
    .replace(/\[[x ]\]\s*/g, "")
    // Remove any remaining Markdown-specific markers
    .replace(/[_~`#*\[\]()]/g, "");

  // Match actual words (consecutive letters and numbers)
  // that aren't just markdown symbols
  const wordMatches = strippedText.match(/\b[\w'-]+\b/g);

  return wordMatches ? wordMatches.length : 0;
}

export function countCharacters(text: string): number {
  return text ? text.length : 0;
}

// Count lines in text
export function countLines(text: string): number {
  if (!text || text === "") return 0;
  return text.split("\n").length;
}

// Estimate reading time based on words
export function estimateReadingTime(text: string): number {
  // Average reading speed: 200 words per minute
  const words = countWords(text);
  return Math.ceil(words / 200);
}

// Format markdown text to HTML for preview
export function formatMarkdown(
  text: string,
  tabColor: string = "#333"
): string {
  if (!text || !text.trim()) return "";

  const unsafeHtml = marked.parse(text, { async: false });

  const safeHtml = DOMPurify.sanitize(unsafeHtml);

  // Generate heading colors based on the tabColor
  const heading1Color = getHeadingColorWithOpacity(tabColor, 1);
  const heading2Color = getHeadingColorWithOpacity(tabColor, 2);
  const heading3Color = getHeadingColorWithOpacity(tabColor, 3);
  const heading4Color = getHeadingColorWithOpacity(tabColor, 4);
  const heading5Color = getHeadingColorWithOpacity(tabColor, 5);
  const heading6Color = getHeadingColorWithOpacity(tabColor, 6);

  // We'll also use the tabColor for some accent elements
  const listMarkerColor = tabColor;
  const linkColor = tabColor;

  // Generate a semi-transparent version of tabColor for horizontal rules
  const hrColor = getHRColor(tabColor);

  // Generate code block colors
  const codeBackground = getCodeBackground(tabColor);
  const codeBorder = getCodeBorder(tabColor);
  const codeAccent = tabColor;

  const css = `
    .markdown-content {
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
      line-height: 1.3;
      color: {$uiColors.textPrimary};
      background-color: {$uiColors.bgPrimary};
      max-width: 900px;
      margin: 0 auto;
      padding: 8px;
    }
    
    /* Ultra-compact headings */
    .markdown-content h1, 
    .markdown-content h2, 
    .markdown-content h3, 
    .markdown-content h4, 
    .markdown-content h5, 
    .markdown-content h6 {
      margin-top: 0.6em;
      margin-bottom: 0.2em;
      line-height: 1.2;
    }
    
    .markdown-content h1 {
      font-size: 1.8em;
      color: ${heading1Color};
      border-bottom: 2px solid ${hrColor};
      padding-bottom: 0.1em;
    }
    
    .markdown-content h2 {
      font-size: 1.5em;
      color: ${heading2Color};
      border-bottom: 1px solid ${hrColor};
      padding-bottom: 0.1em;
    }
    
    .markdown-content h3 {
      font-size: 1.3em;
      color: ${heading3Color};
    }
    
    .markdown-content h4 {
      font-size: 1.1em;
      color: ${heading4Color};
    }
    
    .markdown-content h5 {
      font-size: 1em;
      color: ${heading5Color};
    }
    
    .markdown-content h6 {
      font-size: 0.9em;
      color: ${heading6Color};
    }
    
    /* Ultra-compact paragraph spacing */
    .markdown-content p {
      margin-top: 0em;
      margin-bottom: 0em;
    }
    
    /* Links */
    .markdown-content a {
      color: ${linkColor};
      text-decoration: none;
      border-bottom: 1px dotted {$uiColors.borderLight};
    }
    
    .markdown-content a:hover {
      opacity: 0.8;
      border-bottom: 1px solid {$uiColors.borderMedium};
    }
    
    /* Bold and italic */
    .markdown-content strong {
      font-weight: 600;
      color: ${heading1Color};
    }
    
    .markdown-content em {
      font-style: italic;
      color: {$uiColors.markdown.italicText};
    }
    
    /* Ultra-compact blockquotes */
    .markdown-content blockquote {
      padding: 0.3em 0.6em;
      border-left: 3px solid ${tabColor};
      background-color: {$uiColors.markdown.blockquoteBg};
      margin: 0.4em 0;
      border-radius: 0 2px 2px 0;
      color: {$uiColors.markdown.blockquoteText};
    }
    
    .markdown-content blockquote p {
      margin: 0;
    }
    
    /* Ultra-compact Horizontal rule */
    .markdown-content hr {
      height: 1px;
      border: none;
      background: linear-gradient(to right, transparent, ${hrColor}, transparent);
      margin: 0.6em 0;
    }
    
    /* Ultra-compact lists with minimal spacing */
    .markdown-content ul, 
    .markdown-content ol {
      padding-left: 1.6em;
      margin: 0;
    }
    
    .markdown-content ul {
      list-style-type: none;
    }
    
    .markdown-content ul li,
    .markdown-content ol li {
      margin: 0;
      padding: 0;
      line-height: 1.1;
    }
    
    /* Fix for list items to remove extra spacing */
    .markdown-content li > p {
      margin: 0;
      display: inline;
    }
    
    .markdown-content ul li::before {
      content: "•";
      color: ${listMarkerColor};
      font-weight: bold;            
      padding-right: 1em;
    }
    
    /* Nested lists spacing */
    .markdown-content ul ul,
    .markdown-content ol ol,
    .markdown-content ul ol,
    .markdown-content ol ul {
      margin: 0;
      padding-left: 1em;
    }
    
    .markdown-content p + ul,
    .markdown-content p + ol {
      margin-top: -0.5em;  /* Increased negative margin to pull lists closer to preceding text */
    }

    .markdown-content ul > li:first-child,
    .markdown-content ol > li:first-child {
      margin-top: -0.2em;  /* Pull first list item closer to the list container */
    } 

    /* Ensure consistent spacing within lists */
    .markdown-content li {
      padding-top: 0;
      padding-bottom: 0;
    }

    /* Inline Code styling */
    .markdown-content code {
      font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Courier New', monospace;
      background-color: ${codeBackground};
      color: {$uiColors.textPrimary};
      padding: 0.1em 0.2em;
      border-radius: 2px;
      font-size: 0.9em;
      border-left: 2px solid ${codeAccent};
      letter-spacing: -0.02em;
    }
    
    /* Ultra-compact Code blocks */
    .markdown-content pre {
      background-color: ${codeBackground};
      border-radius: 4px;
      padding: 0;
      overflow: hidden;
      margin: 0.5em 0;
      box-shadow: 0 1px 2px rgba(0,0,0,0.1);
    }
    
    /* Code block header */
    .markdown-content pre::before {
      content: "";
      display: block;
      height: 4px;
      background: linear-gradient(to right, ${codeAccent}, ${getLighterColor(
    codeAccent
  )});
      border-radius: 4px 4px 0 0;
    }
    
    /* Scrollable code content with padding */
    .markdown-content pre > code {
      display: block;
      padding: 0.5em;
      overflow-x: auto;
      background-color: transparent;
      border-left: none;
      box-shadow: none;
      line-height: 1.3;
    }
    
    /* Ultra-compact Tables */
    .markdown-content table {
      width: 100%;
      border-collapse: collapse;
      margin: 0.5em 0;
      overflow: hidden;
      border-radius: 4px;
      border: 1px solid ${getTableBorderColor(tabColor)};
    }
    
    .markdown-content table th {
      background-color: ${getTableHeaderColor(tabColor)};
      font-weight: 600;
      text-align: left;
      color: {$uiColors.textPrimary};
      border-bottom: 1px solid ${getTableBorderColor(tabColor)};
    }
    
    .markdown-content table th,
    .markdown-content table td {
      padding: 0.3em 0.6em;
      border-right: 1px solid ${getTableBorderColor(tabColor)};
    }
    
    .markdown-content table th:last-child,
    .markdown-content table td:last-child {
      border-right: none;
    }
    
    .markdown-content table tr:nth-child(2n) {
      background-color: ${getTableAltRowColor(tabColor)};
    }
    
    /* Images */
    .markdown-content img {
      max-width: 100%;
      height: auto;
      border-radius: 4px;
      margin: 0.4em 0;
    }
    
    /* Task lists */
    .markdown-content input[type="checkbox"] {
      margin: 0 0.2em 0 0;
      vertical-align: middle;
    }
    
    /* Ordered lists with proper numbering */
    .markdown-content ol {
      counter-reset: list-counter;
      list-style-type: none;
    }
    
    .markdown-content ol li {
      counter-increment: list-counter;
      position: relative;
    }
    
    .markdown-content ol li::before {
      content: counter(list-counter) ".";
      position: absolute;
      left: -1.5em;
      font-weight: 500;
      text-align: right;
      color: ${listMarkerColor};
    }
    
    /* Remove extra space at document boundaries */
    .markdown-content > *:first-child {
      margin-top: 0;
    }
    
    .markdown-content > *:last-child {
      margin-bottom: 0;
    }
    
    /* Remove space between elements with specific combinations */
    .markdown-content h1 + p,
    .markdown-content h2 + p,
    .markdown-content h3 + p,
    .markdown-content h4 + p,
    .markdown-content h5 + p,
    .markdown-content h6 + p {
      margin-top: 0;
    }
    
    .markdown-content p + ul,
    .markdown-content p + ol,
    .markdown-content h1 + ul,
    .markdown-content h2 + ul,
    .markdown-content h3 + ul,
    .markdown-content h4 + ul,
    .markdown-content h5 + ul,
    .markdown-content h6 + ul,
    .markdown-content h1 + ol,
    .markdown-content h2 + ol,
    .markdown-content h3 + ol,
    .markdown-content h4 + ol,
    .markdown-content h5 + ol,
    .markdown-content h6 + ol {
      margin-top: 0;
    }
    
    .markdown-content li > ul,
    .markdown-content li > ol {
      margin-top: 0;
    }
  `;

  // Combine the CSS and HTML
  return `
    <style>${css}</style>
    <div class="markdown-content">
      ${safeHtml}
    </div>
  `;
}

// Helper functions remain the same
function getHRColor(tabColor: string): string {
  const color = parseColor(tabColor);
  return `rgba(${color.r}, ${color.g}, ${color.b}, 0.5)`;
}

function getTableHeaderColor(tabColor: string): string {
  const color = parseColor(tabColor);
  const isLight = color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 128;

  if (isLight) {
    return `rgba(${Math.round(color.r * 0.9)}, ${Math.round(
      color.g * 0.9
    )}, ${Math.round(color.b * 0.9)}, 0.2)`;
  } else {
    return `rgba(${Math.min(255, Math.round(color.r * 1.2))}, ${Math.min(
      255,
      Math.round(color.g * 1.2)
    )}, ${Math.min(255, Math.round(color.b * 1.2))}, 0.15)`;
  }
}

function getTableBorderColor(tabColor: string): string {
  const color = parseColor(tabColor);
  return `rgba(${color.r}, ${color.g}, ${color.b}, 0.3)`;
}

function getTableAltRowColor(tabColor: string): string {
  const color = parseColor(tabColor);
  return `rgba(${color.r}, ${color.g}, ${color.b}, 0.05)`;
}

function getTableHoverColor(tabColor: string): string {
  const color = parseColor(tabColor);
  return `rgba(${color.r}, ${color.g}, ${color.b}, 0.1)`;
}

function getCodeBackground(tabColor: string): string {
  const color = parseColor(tabColor);
  const isLight = color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 128;

  if (isLight) {
    return `rgba(${color.r}, ${color.g}, ${color.b}, 0.05)`;
  } else {
    const baseR = Math.round(color.r * 0.2);
    const baseG = Math.round(color.g * 0.2);
    const baseB = Math.round(color.b * 0.2);

    return `rgba(${baseR}, ${baseG}, ${baseB}, 0.95)`;
  }
}

function getCodeBorder(tabColor: string): string {
  const color = parseColor(tabColor);
  return `rgba(${color.r}, ${color.g}, ${color.b}, 0.2)`;
}

function getCodeStringColor(tabColor: string): string {
  const color = parseColor(tabColor);
  const isLight = color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 128;

  if (isLight) {
    return `#38a169`;
  } else {
    return `#68d391`;
  }
}

function getCodeNumberColor(tabColor: string): string {
  const color = parseColor(tabColor);
  const isLight = color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 128;

  if (isLight) {
    return `#9f7aea`;
  } else {
    return `#b794f4`;
  }
}

function getCodeCommentColor(tabColor: string): string {
  const color = parseColor(tabColor);
  const isLight = color.r * 0.299 + color.g * 0.587 + color.b * 0.114 > 128;

  if (isLight) {
    return `#718096`;
  } else {
    return `#a0aec0`;
  }
}

function getLighterColor(color: string): string {
  const parsedColor = parseColor(color);
  const r = Math.min(255, Math.round(parsedColor.r * 1.4));
  const g = Math.min(255, Math.round(parsedColor.g * 1.4));
  const b = Math.min(255, Math.round(parsedColor.b * 1.4));

  return `rgb(${r}, ${g}, ${b})`;
}

function parseColor(color: string): { r: number; g: number; b: number } {
  let r = 0,
    g = 0,
    b = 0;

  if (color.startsWith("#")) {
    if (color.length === 4) {
      r = parseInt(color[1] + color[1], 16);
      g = parseInt(color[2] + color[2], 16);
      b = parseInt(color[3] + color[3], 16);
    } else if (color.length === 7) {
      r = parseInt(color.slice(1, 3), 16);
      g = parseInt(color.slice(3, 5), 16);
      b = parseInt(color.slice(5, 7), 16);
    }
  } else if (color.startsWith("rgb")) {
    const rgbMatch = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (rgbMatch) {
      r = parseInt(rgbMatch[1], 10);
      g = parseInt(rgbMatch[2], 10);
      b = parseInt(rgbMatch[3], 10);
    }
  }

  return { r, g, b };
}
