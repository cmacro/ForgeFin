#!/usr/bin/env python3
"""Convert a Markdown file to HTML and then to PDF using WeasyPrint.
The HTML uses a minimal stylesheet that works for both Latin and Chinese text.
"""

import argparse
import sys
from pathlib import Path

import markdown
from weasyprint import HTML, CSS

# Simple CSS that sets a Chinese-friendly font if available.
CSS_CONTENT = """
/* 页面设置 – 更小的左右边距 */
@page {
    size: A4;
    margin: 1.5cm 0.8cm 1.5cm 0.8cm; /* 上 右 下 左 */
    @bottom-center {
        content: "第 " counter(page) " 页 / 共 " counter(pages) " 页";
        font-size: 9pt;
        color: #555555;
    }
}

/* 字体声明 – 首选微软雅黑 */
@font-face {
    font-family: "ChineseFont";
    src: local('Droid Sans Fallback'), local('Noto Sans CJK SC'), local('WenQuanYi Zen Hei');
    font-weight: normal;
    font-style: normal;
}

body {
    font-family: "Microsoft YaHei", "ChineseFont", "Helvetica", sans-serif;
    line-height: 1.5;
    color: #111827;
    font-size: 11pt; /* 正文字体放大一号 */
    background-color: #ffffff;
    margin: 0;
    padding: 1.0cm 0.6cm; /* 与 @page 边距保持一致 */
}

/* 标题 */
h1 { font-size: 1.8em; margin-top: 1.2em; color:#1f2937; }
h2 { font-size: 1.4em; margin-top: 1em; color:#374151; }
h3 { font-size: 1.2em; margin-top: 0.9em; color:#4b5563; }

/* 代码块 */
code {
    background:#f3f4f6;
    padding:0.15em 0.3em;
    font-family:Courier,monospace;
    font-size: inherit; /* 与所在段落同大小 */
}
pre {
    background:#f3f4f6;
    padding:0.15em 0.3em;
    font-family:Courier,monospace;
    font-size: 9pt; /* 表格内代码仍保持 9pt */
}

/* 表格 */
table { border-collapse:collapse; width:100%; margin:0.8em 0; font-size: 9pt; }
th, td { border:1px solid #d1d5db; padding:0.35em 0.5em; }
th { background:#f3f4f6; }
/* 交替行颜色 */
tr:nth-child(even) td { background:#fafafa; }
"""

def parse_args():
    parser = argparse.ArgumentParser(description="Markdown → HTML → PDF converter using WeasyPrint")
    parser.add_argument("input", help="Markdown source file")
    parser.add_argument("output", help="Resulting PDF file")
    return parser.parse_args()

def main():
    args = parse_args()
    md_path = Path(args.input)
    pdf_path = Path(args.output)

    if not md_path.is_file():
        print(f"Input file not found: {md_path}", file=sys.stderr)
        sys.exit(1)

    md_text = md_path.read_text(encoding="utf-8")
    # Convert markdown to HTML fragment.
    html_body = markdown.markdown(md_text, extensions=["tables", "fenced_code", "toc"])
    # Wrap with a minimal HTML document.
    full_html = f"""<!DOCTYPE html>
<html lang='zh'>
<head>
<meta charset='utf-8'>
<title>{md_path.stem}</title>
<style>{CSS_CONTENT}</style>
</head>
<body>
{html_body}
</body>
</html>"""
    # Render to PDF.
    html_obj = HTML(string=full_html, base_url=str(md_path.parent))
    css_obj = CSS(string=CSS_CONTENT)
    html_obj.write_pdf(target=str(pdf_path), stylesheets=[css_obj])
    print(f"PDF generated: {pdf_path.resolve()}")

if __name__ == "__main__":
    main()
