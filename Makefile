# 变量定义
APP_NAME = ForgeFin 
PORT = 5175

# 默认目标：启动开发环境
all: dev

# -----------------------------------------------------------------------------
# 1. 开发与调试 (Development)
# -----------------------------------------------------------------------------

# 快速检查代码正确性 (无编译)
check:
	cargo check

# 启动 Tauri 开发模式 (包含 Trunk serve)
dev:
	cargo tauri dev

# 仅启动前端服务 (Trunk)
dev-frontend:
	trunk serve --port $(PORT)

# 编译前端生产环境
build-frontend:
	trunk build

# 编译全量应用
build:
	cargo tauri build

# -----------------------------------------------------------------------------
# 2. 数据库管理 (Database)
# -----------------------------------------------------------------------------

# 初始化数据库 (假设后续我们会创建 init_db.sql)
db-init:
	@if [ -f init_db.sql ]; then \
		sqlite3 matflow.db < init_db.sql; \
		echo "Database initialized successfully."; \
	else \
		echo "Error: init_db.sql not found!"; \
	fi

# 打开数据库浏览 (macOS 假设安装了 sqlite3 命令行)
db-shell:
	sqlite3 matflow.db

# 备份数据库
db-backup:
	cp matflow.db matflow_backup_$(shell date +%Y%m%d).db

# -----------------------------------------------------------------------------
# 3. 维护与清理 (Maintenance)
# -----------------------------------------------------------------------------

# 清理所有编译产物
clean:
	cargo clean
	rm -rf dist
	rm -rf src-tauri/target

# 强制杀死所有占用端口的进程 (macOS/Linux)
kill-port:
	lsof -ti :$(PORT) | xargs -r kill -9

# 全量清理运行环境
reset: kill-port clean

# -----------------------------------------------------------------------------
# 辅助工具
# -----------------------------------------------------------------------------

help:
	@echo "MatFlow 开发者快捷指令集:"
	@echo "  make check       - 快速检查代码编译正确性"
	@echo "  make dev          - 启动 Tauri 全量开发环境 (推荐)"
	@echo "  make dev-frontend - 仅启动 Trunk 前端服务"
	@echo "  make build        - 编译发布版本"
	@echo "  make db-init      - 初始化 SQLite 数据库"
	@echo "  make db-shell     - 进入数据库交互命令行"
	@echo "  make kill-port    - 杀死占用 $(PORT) 端口的进程"
	@echo "  make clean        - 清理编译产物"
	@echo "  make reset        - 杀死进程并清理产物 (解决启动报错的神药)"
