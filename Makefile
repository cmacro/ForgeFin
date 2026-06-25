# 变量定义
APP_NAME = ForgeFin 
PORT = 5175

# -----------------------------------------------------------------------------
# 0. 依赖安装 (Dependencies)
# -----------------------------------------------------------------------------

deps:
	npm install

build-css:
	npx tailwindcss -i styles.css -o dist/styles.css

watch-css:
	npx tailwindcss -i styles.css -o dist/styles.css --watch

# 默认目标：启动开发环境
all: build-css dev

# -----------------------------------------------------------------------------
# 1. 开发与调试 (Development)
# -----------------------------------------------------------------------------

check:
	cargo check

dev:
	cargo tauri dev

dev-frontend:
	trunk serve --port $(PORT)

build-frontend:
	trunk build

build:
	cargo tauri build

# -----------------------------------------------------------------------------
# 2. 数据库管理 (Database)
# -----------------------------------------------------------------------------

db-init:
	@if [ -f init_db.sql ]; then \
		sqlite3 matflow.db < init_db.sql; \
		echo "Database initialized successfully."; \
	else \
		echo "Error: init_db.sql not found!"; \
	fi

db-shell:
	sqlite3 matflow.db

db-backup:
	cp matflow.db matflow_backup_$(shell date +%S)

# -----------------------------------------------------------------------------
# 3. 维护与清理 (Maintenance)
# -----------------------------------------------------------------------------

clean:
	cargo clean
	rm -rf dist
	rm -rf src-tauri/target

kill-port:
	lsof -ti :$(PORT) | xargs -r kill -9

reset: kill-port clean

# -----------------------------------------------------------------------------
# 辅助工具
# -----------------------------------------------------------------------------

help:
	@echo "ForgeFin 开发者快捷指令集:"
	@echo "  make deps         - 安装前端依赖 (Tailwind CSS)"
	@echo "  make build-css    - 编译 Tailwind CSS"
	@echo "  make watch-css    - 监听并编译 CSS"
	@echo "  make check       - 快速检查代码编译正确性"
	@echo "  make dev          - 启动 Tauri 全量开发环境"
	@echo "  make build        - 编译发布版本"
	@echo "  make db-init      - 初始化 SQLite 数据库"
	@echo "  make db-shell     - 进入 SQLite 命令行"
	@echo "  make reset        - 清理并重置环境"
