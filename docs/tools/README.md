# 📄 `md_to_html_pdf.py` 使用说明

本文档为 **Markdown → HTML → PDF** 转换脚本提供完整的使用指南，帮助你快速生成带有中文友好排版的 PDF 文件，适用于对外交流、报告、需求说明等场景。

---

## 1. 目的
- 将任意 `.md`（Markdown）文档渲染成 PDF，保证中文字符不出现方框。
- 自动应用统一的页面布局、页眉/页脚、标题层级、表格交替行颜色、代码块样式等。
- 只需一次命令即可完成转换，便于在项目中共享或批量处理文档。

---

## 2. 前置条件
| 项目 | 说明 |
|------|------|
| **Python** | 3.8+（本项目使用 Python 3.11） |
| **WeasyPrint** | 用于把 HTML 渲染成 PDF（会自动安装依赖） |
| **Markdown** | `markdown` 包用于将 Markdown 转为 HTML |
| **系统中文字体** | 脚本会尝试加载以下字体（顺序）<br> - `Microsoft YaHei`<br> - `Droid Sans Fallback`<br> - `Noto Sans CJK SC`<br> - `WenQuanYi Zen Hei`<br>如果系统缺失这些字体，请自行安装（Linux 可用 `apt install fonts-noto-cjk fonts-wqy-microhei`，Windows 自带 YaHei）。 |

> **Tip**：如果在虚拟环境或系统中已经安装 `weasyprint`，直接跳到第 3 步。

---

## 3. 环境搭建（一次性）
> 建议使用 **Python 虚拟环境**，避免依赖冲突。
```bash
# 1️⃣ 在项目根目录创建虚拟环境（可自行命名）
python -m venv .venv

# 2️⃣ 激活虚拟环境
# Linux/macOS
source .venv/bin/activate
# Windows (PowerShell)
.\.venv\Scripts\Activate.ps1

# 3️⃣ 安装所需库
pip install --upgrade pip
pip install weasyprint markdown
```

> **Note**：WeasyPrint 依赖 `cairo`、`Pango`、`GDK‑Pixbuf` 等系统库。
> - **Ubuntu/Debian**：`sudo apt-get install libpango-1.0-0 libpangocairo-1.0-0 libcairo2 libgdk-pixbuf2.0-0`
> - **Fedora**：`sudo dnf install cairo pango gdk-pixbuf2`
> - **macOS**（Homebrew）：`brew install cairo pango gdk-pixbuf`

执行完上述步骤后，运行 `python -c "import weasyprint; print('OK')"` 可确认安装成功。

---

## 4. 脚本参数
```bash
python docs/tools/md_to_html_pdf.py <INPUT.md> <OUTPUT.pdf>
```
| 参数 | 必填 | 说明 |
|------|------|------|
| `<INPUT.md>` | ✅ | 待转换的 Markdown 文件路径。 |
| `<OUTPUT.pdf>` | ✅ | 生成的 PDF 文件路径（可以自定义文件名或目录）。 |

> 脚本仅接受这两个必填位置参数，所有样式均通过内部的 `CSS_CONTENT` 常量控制（见脚本源码）。

---

## 5. 示例
### 5.1 基本使用
```bash
python docs/tools/md_to_html_pdf.py docs/业务数据源分析.md docs/业务数据源分析.pdf
```
> 生成的 PDF 将保存为 `docs/业务数据源分析.pdf`，页眉包含页码，表格、代码块、标题均已美化。

### 5.2 批量转换（示例 Bash 循环）
```bash
#!/usr/bin/env bash
for md in docs/*.md; do
    pdf="${md%.md}.pdf"
    python docs/tools/md_to_html_pdf.py "$md" "$pdf"
    echo "✅ $md → $pdf"
done
```

### 5.3 自定义输出目录
```bash
mkdir -p output/pdfs
python docs/tools/md_to_html_pdf.py docs/业务数据源分析.md output/pdfs/业务数据源分析.pdf
```

---

## 6. 样式（CSS）快速调节
所有外观均在脚本顶部的 `CSS_CONTENT` 变量里定义。常见调节点：
| 区域 | 关键 CSS | 如何修改 |
|------|----------|----------|
| **页边距** | `@page { margin: … }` | 改成 `margin: 1.2cm 0.6cm 1.2cm 0.6cm;`（上、右、下、左） |
| **正文字号** | `body { font-size: 11pt; }` | 改为 `font-size: 12pt;` |
| **表格字号** | `table { font-size: 9pt; }` | 调大为 `10pt` 或更小。 |
| **代码块（块级）** | `pre { font-size: 9pt; }` | 与表格字号保持一致或自行设定。 |
| **行内代码** | `code { font-size: inherit; }` | 如果想让行内代码独立大小，可改为 `font-size: 10pt;` |
| **标题颜色** | `h1 { color:#1f2937; }`、`h2`、`h3` | 替换为项目品牌色，如 `#0d47a1`。 |
| **交替行颜色** | `tr:nth-child(even) td { background:#fafafa; }` | 更改为 `#f0f0f0` 等浅色。 |
| **页眉/页脚** | `@bottom-center { … }` | 可在 `@page` 中添加 `@top-center` 来放置标题或公司 Logo（使用 `content: "Your Title"`）。 |

> 修改后直接保存脚本并重新运行即可看到效果，无需重新安装依赖。

---

## 7. 常见问题 & 调试
| 症状 | 可能原因 | 解决办法 |
|------|----------|----------|
| **中文显示为方框** | 系统未安装的中文字体 | 安装 `Microsoft YaHei`（Windows）或 `fonts-noto-cjk`（Linux）并确保 `@font-face` 能找到。 |
| **WeasyPrint 报错 `cairo`/`pango` 缺失** | 系统库未安装 | 按第 3 步的系统依赖指令安装对应库。 |
| **生成的 PDF 空白** | `markdown` 转换失败（文件路径错误） | 确认输入文件路径正确、文件非空。 |
| **表格宽度超出页面** | 表格列过多且 `width:100%` 仍溢出 | 在 Markdown 中使用 `|` 对齐或在 CSS 中加入 `table { table-layout:fixed; }` 并为列设定 `max-width`。 |
| **代码块颜色不对** | 浏览器默认主题（系统暗色）影响 | PDF 渲染使用硬编码的 CSS，不受系统主题影响；如仍异常，请检查是否编辑了 `code`/`pre` 的 `background` 属性。 |

**调试技巧**
- 在脚本中加入 `print(full_html)`（临时）查看生成的 HTML，确认 Markdown 正确转换。
- 使用 `weasyprint` 命令行调试：`weasyprint file.html out.pdf`（先手动生成 HTML 再渲染）。

---

## 8. 版权 & 贡献
- 本脚本遵循 **MIT License**（默认与项目代码相同的许可）。
- 若需要在其他项目中复用，只要保留文件头部的版权声明即可。
- 欢迎提交 **Pull Request** 改进样式或增加新功能（例如自定义页眉、目录生成等）。

---

## 9. 完整脚本回顾（供快速复制）
```python
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
```
---

### 🎉 就这么简单！
只需 **一步安装** + **一次调用**，即可把任何 Markdown 文档输出为美观、中文友好的 PDF，直接用于对外沟通或存档。祝使用愉快！