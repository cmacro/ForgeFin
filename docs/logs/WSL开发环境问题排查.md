# WSL 下 `cargo tauri dev` 卡在 "Waiting for frontend dev server"

## 现象

在 WSL 环境中运行 `cargo tauri dev`，trunk 正常启动编译，但 Tauri CLI 反复输出：

```
Warn Waiting for your frontend dev server to start on http://127.0.0.1:5175/...
```

无限等待，永不进入下一步，Webview 窗口不出现。

## 环境

- 系统：WSL2 (Ubuntu)
- 框架：Tauri 2.x + Trunk 0.21.14 (Leptos WASM)
- 配置：`beforeDevCommand: "trunk serve --port 5175"`, `devUrl: "http://127.0.0.1:5175"`

## 根本原因

Tauri CLI 在执行 `beforeDevCommand` 后会通过 HTTP 健康检查（GET 请求 `devUrl`）确认前端服务器就绪，然后才启动 Webview 窗口。卡住的常见原因：

### 1. WASM 编译耗时过长

Trunk 首次编译依赖（如 `lucide-leptos`）需要较长时间，Tauri CLI 在 trunk 尚未输出 "serving" 时就已开始健康检查轮询。如果编译时间过长，某些版本下健康检查会进入异常状态。

### 2. 地址绑定/解析不匹配

WSL 环境下，`127.0.0.1` 与 `localhost` 的解析行为可能不同。trunk 默认绑定 `127.0.0.1`，而 Tauri CLI 内部健康检查可能使用不同的地址解析逻辑，导致握手失败。

### 3. 端口转发问题（WSL2 特有）

WSL2 使用虚拟网络适配器，Linux 侧 `127.0.0.1` 与 Windows 侧 `127.0.0.1` 不完全等同。虽然大多数情况能互通，但在某些网络配置下可能出现健康检查无法到达 trunk 服务的情况。

## 解决方案

### 方案一（推荐）：分离前后端启动

绕过 Tauri 的健康检查机制，手动控制启动顺序：

```bash
# 终端 1：启动 trunk
trunk serve --port 5175 --address 0.0.0.0

# 等 trunk 输出 "serving" 日志后，终端 2：
TAURI_DEV_SERVER_URL=http://127.0.0.1:5175 cargo tauri dev
```

`TAURI_DEV_SERVER_URL` 环境变量会告诉 Tauri "前端已经启动好了，直接连接"，跳过 `beforeDevCommand` 和健康检查。

### 方案二：修改绑定地址

2026-07-06 应用配置修改：

- `beforeDevCommand`: `"trunk serve --port 5175"` → `"trunk serve --port 5175 --address 0.0.0.0"`
- `devUrl`: `"http://127.0.0.1:5175"` → `"http://localhost:5175"`

效果：trunk 监听所有接口，Tauri 通过 `localhost` 访问，减少地址解析歧义。

### 方案三：检查 trunk 是否正常启动

在 trunk 编译完成后手动验证服务状态：

```bash
curl -v http://127.0.0.1:5175 2>&1 | head -20
```

预期返回 HTML（Leptos 应用页面）。若 curl 不通，说明 trunk 未正确绑定或端口被占用。

## 验证结果

采用方案二修改后，`cargo tauri dev` 正常通过健康检查，Trunk 编译完成后 Tauri 窗口成功启动。
