use leptos::prelude::*;

use crate::components::source::raw_record_table::RawRecordTable;
use crate::components::source::record_detail::RecordDetail;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, RawRecordFilter};

#[component]
pub fn ChatRecords() -> impl IntoView {
    let (selected_id, set_selected_id) = signal(Option::<i64>::None);

    let records = LocalResource::new(|| async move {
        ipc::list_raw_records(&RawRecordFilter {
            source_type: Some("chat_record".to_string()),
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
            <div class="card p-4 mb-4">
                <p class="text-13 text-secondary">
                    "本页面显示从微信群获取的上报单据信息，包含原始凭证照片、发送人、消费或收入项目。请先在「导入中心」导入聊天记录数据。"
                </p>
            </div>

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
                                Err(_e) => view! {
                                    <div class="empty-state">
                                        <p class="empty-state-desc">"暂无聊天记录数据，请在「原始凭证 > 导入中心」中导入。"</p>
                                    </div>
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
