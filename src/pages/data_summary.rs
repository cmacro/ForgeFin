use leptos::prelude::*;

use crate::components::source::raw_record_table::RawRecordTable;
use crate::components::source::record_detail::RecordDetail;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, RawRecordFilter};

#[component]
pub fn DataSummary() -> impl IntoView {
    let (selected_id, set_selected_id) = signal(Option::<i64>::None);

    let records = LocalResource::new(|| async move {
        ipc::list_raw_records(&RawRecordFilter {
            source_type: Some("summary_flow".to_string()),
            batch_id: None,
            page: 1,
            page_size: 50,
        })
        .await
    });

    let detail = LocalResource::new(move || {
        let id = selected_id.get();
        async move {
            match id {
                Some(id) => ipc::get_raw_record(id).await.unwrap_or(None),
                None => None,
            }
        }
    });

    view! {
        <div class="page-content">
            <div class="page-grid-detail">
                <div class="flex flex-col min-h-0">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || Suspend::new(async move {
                            match records.await {
                                Ok(p) => view! {
                                    <RawRecordTable
                                        rows=p.items.clone()
                                        selected_id=selected_id
                                        set_selected_id=set_selected_id
                                    />
                                    <div class="border-t border-border-light">
                                        <Pagination total=p.total current=p.page page_size=p.page_size />
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="login-error">{format!("加载失败: {e}")}</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
                <div class="detail-panel-responsive">
                    <RecordDetail detail=detail />
                </div>
            </div>
        </div>
    }
}
