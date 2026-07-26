#!/usr/bin/env python3
"""Generate 2 months of health company sample data (2026-05-01 to 2026-06-30)."""

import random
from datetime import datetime, timedelta
from pathlib import Path

random.seed(20260501)

# ---------- Constants ----------
MERCHANT_NAME = "锦绣健康管理有限公司"
MERCHANT_NO = "100172984575"
PRODUCT_NAME = "线下产品"
AGREEMENT_NAME = "锦绣健康管理有限公司"
AGREEMENT_NO = "0301"
BIZ_NO = "1001****3418"

CUSTOMERS = ["王浩然", "赵雨桐", "刘思琪", "吴佳怡", "李娜娜"]

COUNTERPARTY_COMPANIES = [
    "明瑞科技有限公司", "安泊酒店管理公司", "恒悦医疗科技有限公司",
    "瑞康医疗器械有限公司", "泰禾医疗器械有限公司",
]

SUMMARY_PROJECTS_INCOME = ["营业收入"]
SUMMARY_PROJECTS_EXPENSE = [
    "产品成本", "护理成本", "水电费", "房租", "工资", "营销费用",
    "办公用品", "维修费", "手续费",
]

EXPENSE_REASONS = {
    "产品成本": ["货款", "原材料采购", "产品进货"],
    "护理成本": ["报销", "耗材采购", "护理用品"],
    "水电费": ["电费", "水费", "物业费"],
    "房租": ["门店房租", "办公室房租"],
    "工资": ["员工工资", "社保缴费", "公积金"],
    "营销费用": ["广告投放", "推广费", "活动费"],
    "办公用品": ["办公耗材", "打印机耗材", "文具"],
    "维修费": ["设备维修", "空调维修", "水电维修"],
    "手续费": ["跨行汇款手续费", "账户管理费", "网银费"],
}

EXPENSE_TARGETS = {
    "产品成本": COUNTERPARTY_COMPANIES,
    "护理成本": ["李娜娜", "王浩然", "赵雨桐"],
    "水电费": ["安泊酒店管理公司", "明瑞科技有限公司"],
    "房租": ["安泊酒店管理公司"],
    "工资": ["李娜娜", "王浩然", "赵雨桐", "刘思琪", "吴佳怡"],
    "营销费用": ["明瑞科技有限公司", "恒悦医疗科技有限公司"],
    "办公用品": ["恒悦医疗科技有限公司", "泰禾医疗器械有限公司"],
    "维修费": ["恒悦医疗科技有限公司"],
    "手续费": [],
}

# 微信/支付宝 渠道费率 0.25%
FEE_RATE_WECHAT_ALIPAY = 0.0025

# ---------- Helpers ----------
def fmt_amt(x: float) -> str:
    return f"{x:,.2f}"

def dt_str(dt: datetime) -> str:
    return dt.strftime("%Y-%m-%d %H:%M:%S")

def date_str(dt: datetime) -> str:
    return dt.strftime("%Y-%m-%d")

def random_time_on_day(day: datetime, hour_range=(8, 22), second=None) -> datetime:
    h = random.randint(hour_range[0], hour_range[1] - 1)
    m = random.randint(0, 59)
    s = second if second is not None else random.randint(0, 59)
    return day.replace(hour=h, minute=m, second=s)

def random_workday_amount() -> float:
    """Generate realistic order amount (health company services)."""
    # 服务/产品类金额
    bucket = random.random()
    if bucket < 0.30:
        return round(random.uniform(100, 500), 2)
    elif bucket < 0.55:
        return round(random.uniform(500, 1500), 2)
    elif bucket < 0.80:
        return round(random.uniform(1500, 5000), 2)
    elif bucket < 0.95:
        return round(random.uniform(5000, 12000), 2)
    else:
        return round(random.uniform(12000, 20000), 2)

def is_workday(dt: datetime) -> bool:
    return dt.weekday() < 5

# ---------- Main ----------
def main():
    start = datetime(2026, 5, 1)
    end = datetime(2026, 6, 30)
    n_days = (end - start).days + 1  # 61 days

    # Daily counts (weekends lighter)
    def daily_order_count(d):
        base = 38 if is_workday(d) else 18
        return base + random.randint(-3, 5)

    # Generate per-day data
    all_orders = []   # (datetime, order_id, amount, fee, net, customer, pay_method)
    all_pos = []      # (datetime, order_id, amount, status)
    all_bank = []     # list of dicts
    all_summary = []  # list of dicts

    order_seq = 100000
    receipt_seq = 600
    balance = 650000.00  # initial balance

    # Collect orders first, then derive bank / pos / summary
    for d_offset in range(n_days):
        day = start + timedelta(days=d_offset)
        n_orders = daily_order_count(day)
        day_orders = []
        for _ in range(n_orders):
            order_seq += 1
            t = random_time_on_day(day, hour_range=(9, 21))
            amount = random_workday_amount()
            fee = round(amount * FEE_RATE_WECHAT_ALIPAY, 2)
            net = round(amount - fee, 2)
            customer = random.choice(CUSTOMERS)
            pay_method = random.choices(
                ["微信", "支付宝"], weights=[0.7, 0.3]
            )[0]
            card_type = random.choices(
                ["借记卡", "贷记卡"], weights=[0.7, 0.3]
            )[0]
            # 工行订单号格式: 1001729845750000 + YY MM DD + HH MM SS + seq
            # 取时间后 6 位 HHMMSS
            order_id = (
                f"{MERCHANT_NO}0000"
                f"{t.strftime('%y%m%d%H%M%S')}"
                f"{order_seq % 100000:05d}"
            )
            # 商户订单号 (empty in source for most)
            biz_order_no = ""
            third_party_no = (
                f"45{random.randint(10**16, 10**17 - 1)}" if pay_method == "微信"
                else f"79{random.randint(10**16, 10**17 - 1)}"
            )
            consume_discount = round(random.choice([0, 0, 0, 0, 0.10, 0.20, 0.50, 1.00, 2.00, 3.00]), 2)
            day_orders.append({
                "time": t,
                "order_id": order_id,
                "biz_order_no": biz_order_no,
                "amount": amount,
                "fee": fee,
                "net": net,
                "customer": customer,
                "pay_method": pay_method,
                "card_type": card_type,
                "third_party_no": third_party_no,
                "consume_discount": consume_discount,
            })
        all_orders.extend(day_orders)

    # ---------- POS raw (one row per order) ----------
    pos_lines = ["交易时间\t协议编号/名称\t工行订单号\t商户订单号\t订单金额\t累计退款金额\t订单状态\t结算状态"]
    for o in all_orders:
        pos_lines.append(
            f"{dt_str(o['time'])}\t{AGREEMENT_NO}{MERCHANT_NAME}\t"
            f"{o['order_id'][:7]}...{o['order_id'][-8:]}\t"
            f"-\t{fmt_amt(o['amount'])}\t0.00\t已付款\t已结算"
        )

    # ---------- Order raw (full columns) ----------
    order_header = (
        "商户名称\t商户编号\t产品名称\t协议名称\t协议编号\t商户订单号\t工行订单号\t"
        "订单金额\t手续费金额\t商户实收金额\t累计退款金额\t交易时间\t支付时间\t"
        "交易类型\t订单状态\t收单接入方式\t支付方式\t借贷记标识\t客户备注\t"
        "入账账户类型\t入账账号\t商户优惠\t银行补贴\t工银i豆抵扣金额\t"
        "电子券银行补贴\t团购券银行补贴\t分期金额\t分期期数\t分期商户贴息\t"
        "交易卡号\t卡种\t第三方订单号\t电子券抵扣金额\t交易流水号\t"
        "商户补贴金额\t消费立减金额\t终端编号\t结算状态\t结算日期"
    )
    order_lines = [order_header]
    for o in all_orders:
        pay_time = o["time"] + timedelta(seconds=random.randint(5, 15))
        settle_date = o["time"] + timedelta(days=1)
        order_lines.append(
            f"{MERCHANT_NAME}\t{MERCHANT_NO}\t{PRODUCT_NAME}\t{AGREEMENT_NAME}\t"
            f"{AGREEMENT_NO}\t{o['biz_order_no']}\t{o['order_id']}\t"
            f"{o['amount']:.2f}\t{o['fee']:.2f}\t{o['net']:.2f}\t0.00\t"
            f"{dt_str(o['time'])}\t{dt_str(pay_time)}\t消费\t已付款\t主扫码\t"
            f"{o['pay_method']}\t{o['card_type']}\t{o['customer']}\t"
            f"往来账户\t{BIZ_NO}\t0.00\t0.00\t0.00\t0.00\t0.00\t0.00\t0.00\t"
            f"0.00\t\t{o['card_type']}\t{o['third_party_no']}\t0.00\t"
            f"{o['order_id']}\t0.00\t{fmt_amt(o['consume_discount'])}\t0\t"
            f"已结算\t{date_str(settle_date)}"
        )

    # ---------- Bank raw ----------
    # Aggregate orders by settle date (next day) to produce POS clearing rows.
    # Keep orders in a dict by settle date.
    from collections import defaultdict
    by_settle = defaultdict(list)
    for o in all_orders:
        settle_date = (o["time"] + timedelta(days=1)).date()
        by_settle[settle_date].append(o)

    bank_lines = ["凭证号\t对方账号\t交易时间\t对方单位\t用途\t摘要\t附言\t转入金额\t转出金额\t余额"]
    pos_clearing_balance = 600000.00  # arbitrary opening

    # Build transactions in chronological order
    bank_events = []  # (datetime, kind, amount, target, memo, abstract)

    # POS clearing: produce per-order individual clearing entries
    # (realistic for a busy month; matches the 1-to-1 reconciliation flow)
    for o in all_orders:
        clear_time = (o["time"] + timedelta(days=1)).replace(
            hour=2, minute=random.randint(40, 55), second=random.randint(0, 59)
        )
        bank_events.append({
            "time": clear_time,
            "kind": "pos_clearing",
            "amount_in": o["net"],
            "target": "待清算商户银行卡POS净收暂收销账户",
            "purpose": "",
            "abstract": "",
            "memo": MERCHANT_NO,
            "account": "1001180011200143186",
        })

    # Generate expense transactions (per day, several types)
    for d_offset in range(n_days):
        day = start + timedelta(days=d_offset)
        n_expenses = random.randint(3, 7) if is_workday(day) else random.randint(1, 3)
        for _ in range(n_expenses):
            project = random.choices(
                list(EXPENSE_REASONS.keys()),
                weights=[3, 5, 2, 1, 2, 2, 2, 1, 5],
            )[0]
            reason = random.choice(EXPENSE_REASONS[project])
            target_pool = EXPENSE_TARGETS[project]
            target = random.choice(target_pool) if target_pool else ""
            if project in ("产品成本", "水电费", "房租", "营销费用", "办公用品", "维修费") and target:
                account_no = f"{random.randint(10**15, 10**16 - 1)}"
            elif project in ("护理成本", "工资") and target:
                account_no = "6226220219453718"
            elif project == "手续费":
                account_no = "1001211311500819234"
            else:
                account_no = ""
            if project == "工资":
                amount = round(random.uniform(3000, 12000), 2)
            elif project == "房租":
                amount = round(random.uniform(15000, 60000), 2)
            elif project == "产品成本":
                amount = round(random.uniform(800, 15000), 2)
            elif project == "营销费用":
                amount = round(random.uniform(500, 8000), 2)
            elif project == "手续费":
                amount = round(random.choice([9, 15, 18, 25, 31.5, 45]), 2)
            else:
                amount = round(random.uniform(50, 3000), 2)
            t = random_time_on_day(day, hour_range=(9, 22))
            bank_events.append({
                "time": t,
                "kind": "expense",
                "amount_out": amount,
                "target": target,
                "purpose": reason,
                "abstract": reason,
                "memo": "",
                "account": account_no,
                "project": project,
                "reason": reason,
            })

        # 拨款: 偶发, 1-2 次/月
        if day.day in (1, 15) and random.random() < 0.5:
            t = random_time_on_day(day, hour_range=(10, 14))
            bank_events.append({
                "time": t,
                "kind": "transfer_in",
                "amount_in": round(random.uniform(50000, 200000), 2),
                "target": MERCHANT_NAME,
                "purpose": "拨款",
                "abstract": "拨款",
                "memo": "",
                "account": "160787593",
            })

    # Sort and walk balance
    bank_events.sort(key=lambda e: e["time"])
    cur_balance = 762863.23  # opening (matches original last balance)
    for ev in bank_events:
        if ev["kind"] == "pos_clearing":
            cur_balance = round(cur_balance + ev["amount_in"], 2)
            bank_lines.append(
                f"000000000\t{ev['account']}\t{dt_str(ev['time'])}\t"
                f"{ev['target']}\t{ev['purpose']}\t{ev['abstract']}\t{ev['memo']}\t"
                f"{ev['amount_in']:.2f}\t\t{cur_balance:.2f}"
            )
        elif ev["kind"] == "expense":
            cur_balance = round(cur_balance - ev["amount_out"], 2)
            bank_lines.append(
                f"000000000\t{ev['account']}\t{dt_str(ev['time'])}\t"
                f"{ev['target']}\t{ev['purpose']}\t{ev['abstract']}\t{ev['memo']}\t"
                f"\t{ev['amount_out']:.2f}\t{cur_balance:.2f}"
            )
        elif ev["kind"] == "transfer_in":
            cur_balance = round(cur_balance + ev["amount_in"], 2)
            bank_lines.append(
                f"000000000\t{ev['account']}\t{dt_str(ev['time'])}\t"
                f"{ev['target']}\t{ev['purpose']}\t{ev['abstract']}\t{ev['memo']}\t"
                f"{ev['amount_in']:.2f}\t\t{cur_balance:.2f}"
            )

    # ---------- Summary raw ----------
    summary_header = (
        "日期\t收据编号\t一级\t项目\t事由\t借记卡手续费\t信用卡手续费\t"
        "微信/支付宝\t手续费\t实际收入\t支出\t余额\t备注"
    )
    summary_lines = [summary_header]
    # Sub-header row matching original
    summary_lines.append(
        "\t\t\t\t\t0.005\t0.006\t0.0025\t\t\t\t收支"
    )

    # Build summary from orders + bank expenses, ordered by date
    summary_events = []
    # Income from orders
    for o in all_orders:
        summary_events.append({
            "date": o["time"].date(),
            "time": o["time"],
            "kind": "income",
            "project": "营业收入",
            "reason": f"{o['customer']}{random.choice(['产康充值', '牛奶光', 'Indiba', '美肤炫', '护理套餐', '产品销售', '面部护理', '身体SPA'])}",
            "amount_in": o["amount"],
            "fee": o["fee"],
            "net": o["net"],
            "target": "待清算商户银行卡POS净收暂收销账户",
            "customer": o["customer"],
        })
    # Expense events
    for ev in bank_events:
        if ev["kind"] == "expense":
            summary_events.append({
                "date": ev["time"].date(),
                "time": ev["time"],
                "kind": "expense",
                "project": ev["project"],
                "reason": ev["reason"],
                "amount_out": ev["amount_out"],
                "target": ev["target"],
            })

    # Sort by date then time
    summary_events.sort(key=lambda e: (e["date"], e.get("time", datetime.min)))

    # Walk balance per day, accumulate
    daily_balance = {}
    bal = 762863.23
    # We use last day's bank balance as ending balance
    last_bal = cur_balance

    # Compute running balance per day
    bank_by_day = defaultdict(lambda: {"in": 0.0, "out": 0.0})
    for ev in bank_events:
        d = ev["time"].date()
        if ev["kind"] in ("pos_clearing", "transfer_in"):
            bank_by_day[d]["in"] += ev["amount_in"]
        elif ev["kind"] == "expense":
            bank_by_day[d]["out"] += ev["amount_out"]

    # Emit summary entries
    for ev in summary_events:
        d = ev["date"]
        is_income = ev["kind"] == "income"
        receipt_no = ""
        if is_income:
            receipt_seq += 1
            receipt_no = f"{receipt_seq:07d}"
        # Balance after event
        cur_bal_after = 0.0  # placeholder, will be set properly below
        # Build line
        if is_income:
            line = (
                f"{date_str(d)}\t{receipt_no}\t收入\t{ev['project']}\t{ev['reason']}\t"
                f"\t\t{fmt_amt(ev['amount_in'])}\t{fmt_amt(ev['fee'])}\t"
                f"{fmt_amt(ev['net'])}\t\t\t{ev['target']}\t7\t{ev['net']:.2f}"
            )
        else:
            line = (
                f"{date_str(d)}\t\t支出\t{ev['project']}\t{ev['reason']}\t"
                f"\t\t\t\t\t"
                f"{ev['amount_out']:.2f}\t\t{ev['target']}\t7\t{ev['amount_out']:.2f}"
            )
        summary_lines.append(line)

    # ---------- Write files ----------
    out_dir = Path("tests/sample_data/health_company")
    out_dir.mkdir(parents=True, exist_ok=True)

    (out_dir / "order_raw.tsv").write_text("\n".join(order_lines) + "\n", encoding="utf-8")
    (out_dir / "pos_raw.tsv").write_text("\n".join(pos_lines) + "\n", encoding="utf-8")
    (out_dir / "bank_raw.tsv").write_text("\n".join(bank_lines) + "\n", encoding="utf-8")
    (out_dir / "summary_raw.tsv").write_text("\n".join(summary_lines) + "\n", encoding="utf-8")

    # ---------- Report ----------
    print(f"Days: {n_days}")
    print(f"Orders: {len(all_orders)}")
    print(f"POS: {len(pos_lines) - 1}")
    print(f"Bank: {len(bank_lines) - 1}")
    print(f"Summary: {len(summary_lines) - 2}")


if __name__ == "__main__":
    main()
