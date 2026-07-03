use leptos::prelude::*;

use crate::components::charts::kpi_card::{KpiAccent, KpiCard};

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <KpiCard label="现金余额" value="¥ 1,280,500.00".to_string() unit=None accent=KpiAccent::Info />
                <KpiCard label="应收账款" value="¥ 420,300.00".to_string() unit=None accent=KpiAccent::Success />
                <KpiCard label="应付账款" value="¥ 310,200.00".to_string() unit=None accent=KpiAccent::Warning />
                <KpiCard label="本月收入" value="¥ 856,000.00".to_string() unit=None accent=KpiAccent::Neutral />
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                <div class="lg:col-span-2 card">
                    <div class="card-header">
                        <h3 class="card-title">"本月收支趋势"</h3>
                    </div>
                    <div class="card-body">
                        <div class="h-48 flex items-end justify-around border-b border-muted">
                            {bars()}
                        </div>
                    </div>
                </div>
                <div class="card">
                    <div class="card-body">
                        <h3 class="text-sm font-medium text-primary mb-3">"待办事项"</h3>
                        <ul class="space-y-2 text-sm">
                            <li class="flex items-center justify-between py-2" style="border-bottom: 1px solid var(--color-border-light)">
                                <span class="text-primary">"待审核凭证"</span>
                                <span class="text-warning font-medium">"12"</span>
                            </li>
                            <li class="flex items-center justify-between py-2" style="border-bottom: 1px solid var(--color-border-light)">
                                <span class="text-primary">"逾期应收"</span>
                                <span class="text-danger font-medium">"5"</span>
                            </li>
                            <li class="flex items-center justify-between py-2" style="border-bottom: 1px solid var(--color-border-light)">
                                <span class="text-primary">"即将到期应付"</span>
                                <span class="text-warning font-medium">"8"</span>
                            </li>
                            <li class="flex items-center justify-between py-2">
                                <span class="text-primary">"月末结账"</span>
                                <span class="text-info font-medium">"3"</span>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn bars() -> Vec<impl IntoView> {
    let heights = [40, 65, 50, 80, 70, 95, 60];
    heights
        .iter()
        .map(|h| {
            view! {
                <div class="w-6 bg-brand rounded-t-sm" style=format!("height: {h}%")></div>
            }
        })
        .collect()
}
