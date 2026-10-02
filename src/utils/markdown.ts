import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js';

const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
  highlight(str: string, lang: string): string {
    if (lang && hljs.getLanguage(lang)) {
      try {
        const highlighted = hljs.highlight(str, { language: lang, ignoreIllegals: true }).value;
        return `<pre class="hljs-code-block"><code class="hljs language-${lang}">${highlighted}</code></pre>`;
      } catch {
        // fall through
      }
    }
    return `<pre class="hljs-code-block"><code>${md.utils.escapeHtml(str)}</code></pre>`;
  },
});

export function renderMarkdown(text: string): string {
  return md.render(text);
}
