/* ============================================================
   ForgeFin · 表格页脚统计（Excel 式区域选择）
   用法：页面只需引入 <script src="grid-stats.js" defer></script>
   - 单元格点击 / 拖拽框选 / Shift 扩选 / Ctrl(⌘)+A 全选 / Esc 清除
   - 页脚实时显示选中区域的合计、平均、数量、最大、最小（按列）
   ============================================================ */
(function () {
  'use strict';

  var NUMERIC = /^[-+]?[\d,]+(\.\d+)?%?$/;
  var state = { table: null, anchor: null, focus: null, dragging: false };

  function parseNumber(text) {
    if (text == null) return null;
    var t = String(text).trim();
    if (!t || t === '-' || t === '—' || t === '/') return null;
    if (!NUMERIC.test(t.replace(/\s/g, ''))) return null;
    var digits = t.replace(/[^\d]/g, '');
    if (digits.length > 11) return null;            // 订单号 / 账号等长数字串不是金额
    if (/^0\d/.test(t.replace(/[\s,]/g, ''))) return null;
    var n = Number(t.replace(/[\s,¥$€]/g, '').replace('%', ''));
    return isFinite(n) ? n : null;
  }

  function formatNumber(n, money) {
    if (!isFinite(n)) return '0';
    var decimals = money || Math.abs(n - Math.round(n)) > 1e-9 ? 2 : 0;
    return n.toLocaleString('zh-CN', {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals
    });
  }

  function cellOf(el) {
    var td = el && el.closest ? el.closest('td') : null;
    if (!td) return null;
    var row = td.parentElement;
    if (!row || row.parentElement.tagName !== 'TBODY') return null;
    return td;
  }

  function coords(td) {
    var table = td.closest('table');
    var row = td.parentElement;
    var tbody = row.parentElement;
    var rows = Array.prototype.slice.call(tbody.rows);
    return { table: table, r: rows.indexOf(row), c: td.cellIndex, td: td };
  }

  function tableFor(td) {
    return td.closest('table');
  }

  function bodyRows(table) {
    var tbody = table.tBodies[0] || table.querySelector('tbody');
    return tbody ? Array.prototype.slice.call(tbody.rows) : [];
  }

  function headerLabel(table, c) {
    var th = table.querySelector('thead tr') && table.querySelector('thead tr').children[c];
    return th ? (th.textContent || '').trim() : '第 ' + (c + 1) + ' 列';
  }

  function bounds() {
    var a = state.anchor, f = state.focus;
    if (!a || !f) return null;
    return {
      r1: Math.min(a.r, f.r), r2: Math.max(a.r, f.r),
      c1: Math.min(a.c, f.c), c2: Math.max(a.c, f.c)
    };
  }

  function clearMarks(table) {
    if (!table) return;
    var sel = table.querySelectorAll('td.cell-sel, td.cell-anchor');
    for (var i = 0; i < sel.length; i++) sel[i].classList.remove('cell-sel', 'cell-anchor');
  }

  function paint(table) {
    clearMarks(table);
    var b = bounds();
    if (!b) return;
    var rows = bodyRows(table);
    for (var r = b.r1; r <= b.r2; r++) {
      var row = rows[r];
      if (!row) continue;
      for (var c = b.c1; c <= b.c2; c++) {
        var td = row.cells[c];
        if (td) td.classList.add('cell-sel');
      }
    }
    if (state.anchor) {
      var arow = rows[state.anchor.r];
      if (arow && arow.cells[state.anchor.c]) arow.cells[state.anchor.c].classList.add('cell-anchor');
    }
  }

  function footerBar(table) {
    var main = table.closest('.main') || document.body;
    var bar = main.querySelector('.status-bar, .page-footer');
    if (!bar) {
      bar = document.createElement('footer');
      bar.className = 'page-footer';
      bar.setAttribute('data-od-id', 'page-footer');
      bar.innerHTML = '<div class="footer-stats"></div><div class="footer-right"></div>';
      main.appendChild(bar);
    }
    bar.classList.add('has-grid-stats');
    var stats = bar.querySelector('.footer-stats');
    if (!stats) {
      stats = document.createElement('div');
      stats.className = 'footer-stats';
      bar.insertBefore(stats, bar.firstChild);
    }
    return stats;
  }

  function render() {
    var table = state.table;
    if (!table || !document.body.contains(table)) {
      var first = document.querySelector('.data-table');
      if (!first) return;
      state.table = first;
      table = first;
    }
    var stats = footerBar(table);
    var b = bounds();
    var rows = bodyRows(table);

    if (!b) {
      stats.innerHTML = '<span class="footer-hint">选中单元格或拖拽框选区域，页脚显示合计 / 平均 / 数量 / 最大 / 最小</span>';
      updateRight(table, 0, rows.length);
      return;
    }

    var chips = [];
    var selectedRows = b.r2 - b.r1 + 1;
    var selectedCols = b.c2 - b.c1 + 1;

    for (var c = b.c1; c <= b.c2; c++) {
      var nums = [];
      var nonEmpty = 0;
      var hasDecimal = false;
      var label = headerLabel(table, c);
      var idCol = /单号|编号|账号|电话|收据号|订单号|批次/.test(label);
      for (var r = b.r1; r <= b.r2; r++) {
        var row = rows[r];
        if (!row || !row.cells[c]) continue;
        var raw = (row.cells[c].textContent || '').trim();
        if (raw) nonEmpty++;
        var n = parseNumber(raw);
        if (n !== null && !idCol) {
          nums.push(n);
          if (raw.indexOf('.') >= 0) hasDecimal = true;
        }
      }
      var moneyCol = hasDecimal || /金额|余额|实收|账面|手续费|差异|收入|支出/.test(label);
      if (nums.length && nums.length >= Math.max(1, Math.ceil(nonEmpty * 0.6))) {
        var sum = 0, max = -Infinity, min = Infinity;
        for (var i = 0; i < nums.length; i++) {
          sum += nums[i];
          if (nums[i] > max) max = nums[i];
          if (nums[i] < min) min = nums[i];
        }
        chips.push(
          '<span class="stat"><span class="stat-label">' + esc(label) + '</span>' +
          '<span class="stat-item">合计 <b class="text-num">' + formatNumber(sum, moneyCol) + '</b></span>' +
          '<span class="stat-item">平均 <b class="text-num">' + formatNumber(sum / nums.length, moneyCol) + '</b></span>' +
          '<span class="stat-item">数量 <b class="text-num">' + nums.length + '</b></span>' +
          '<span class="stat-item">最大 <b class="text-num">' + formatNumber(max, moneyCol) + '</b></span>' +
          '<span class="stat-item">最小 <b class="text-num">' + formatNumber(min, moneyCol) + '</b></span>' +
          '</span>'
        );
      } else if (nonEmpty) {
        chips.push(
          '<span class="stat"><span class="stat-label">' + esc(label) + '</span>' +
          '<span class="stat-item">非空 <b class="text-num">' + nonEmpty + '</b></span></span>'
        );
      }
    }

    stats.innerHTML =
      '<span class="stat-scope">已选 ' + selectedRows + ' 行 × ' + selectedCols + ' 列</span>' +
      (chips.length ? chips.join('') : '<span class="footer-hint">所选区域没有可统计的数值</span>') +
      '<button type="button" class="stat-clear" title="清除选择（Esc）">清除</button>';

    var clearBtn = stats.querySelector('.stat-clear');
    if (clearBtn) clearBtn.addEventListener('click', function () { clearSelection(); });

    updateRight(table, selectedRows, rows.length);
  }

  function updateRight(table, selectedRows, totalRows) {
    var main = table.closest('.main') || document.body;
    var bar = main.querySelector('.status-bar, .page-footer');
    if (!bar) return;
    var right = bar.querySelector('.footer-right');
    if (!right) {
      right = document.createElement('div');
      right.className = 'footer-right';
      bar.appendChild(right);
    }
    var totalEl = right.querySelector('.footer-total');
    if (!totalEl) {
      totalEl = document.createElement('span');
      totalEl.className = 'footer-total';
      right.insertBefore(totalEl, right.firstChild);
    }
    totalEl.textContent = selectedRows
      ? '已选 ' + selectedRows + ' 行 · 共 ' + totalRows + ' 行'
      : '共 ' + totalRows + ' 行';
  }

  function esc(s) {
    return String(s).replace(/[&<>"]/g, function (ch) {
      return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch];
    });
  }

  function clearSelection() {
    var table = state.table;
    state.anchor = null;
    state.focus = null;
    state.dragging = false;
    if (table) clearMarks(table);
    render();
  }

  function startSelection(td, extend) {
    var pos = coords(td);
    if (!pos) return;
    if (state.table !== pos.table) clearMarks(state.table);
    state.table = pos.table;
    if (!extend || !state.anchor) state.anchor = { r: pos.r, c: pos.c };
    state.focus = { r: pos.r, c: pos.c };
    paint(pos.table);
    render();
  }

  document.addEventListener('mousedown', function (e) {
    var td = cellOf(e.target);
    if (!td) return;
    if (e.target.closest('button, a, input, select, textarea')) return;
    if (e.button !== 0) return;
    e.preventDefault();
    state.dragging = true;
    startSelection(td, e.shiftKey);
  });

  document.addEventListener('mouseover', function (e) {
    if (!state.dragging || !state.table) return;
    var td = cellOf(e.target);
    if (!td || td.closest('table') !== state.table) return;
    var pos = coords(td);
    if (!pos) return;
    if (state.focus && state.focus.r === pos.r && state.focus.c === pos.c) return;
    state.focus = { r: pos.r, c: pos.c };
    paint(state.table);
    render();
  });

  document.addEventListener('mouseup', function () {
    if (!state.dragging) return;
    state.dragging = false;
  });

  document.addEventListener('keydown', function (e) {
    var tag = (document.activeElement && document.activeElement.tagName) || '';
    var typing = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
    if (e.key === 'Escape' && state.anchor) {
      if (!typing) clearSelection();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && (e.key === 'a' || e.key === 'A') && !typing) {
      var table = state.table || document.querySelector('.data-table');
      if (!table) return;
      var rows = bodyRows(table);
      if (!rows.length) return;
      e.preventDefault();
      state.table = table;
      state.anchor = { r: 0, c: 0 };
      state.focus = { r: rows.length - 1, c: rows[rows.length - 1].cells.length - 1 };
      paint(table);
      render();
    }
  });

  function watch() {
    var tables = document.querySelectorAll('table.data-table');
    for (var i = 0; i < tables.length; i++) {
      (function (table) {
        if (table.dataset.gridStats) return;
        table.dataset.gridStats = '1';
        var tbody = table.tBodies[0] || table.querySelector('tbody');
        if (!tbody) return;
        new MutationObserver(function () {
          if (state.table === table) {
            state.anchor = null;
            state.focus = null;
          }
          render();
        }).observe(tbody, { childList: true, subtree: true, characterData: true });
      })(tables[i]);
    }
    if (!state.table && tables.length) state.table = tables[0];
    render();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', watch);
  } else {
    watch();
  }
  window.addEventListener('load', watch);
})();
