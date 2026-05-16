# LizzieYzy Next Generation — 重写 Roadmap

## Context

LizzieYzy 当前基于 Java Swing，117k 行代码中 75% 是 UI 渲染逻辑。Swing 无法现代化、无法支持手机端、性能瓶颈明显。目标是重写为一个跨平台（Win/Mac/Linux + 未来手机）的现代围棋引擎 GUI。

---

## 当前进展快照（2026-05-16）

### 已完成

- **项目骨架**：Tauri 2.x + Rust workspace + Svelte 5 前端已经跑通，桌面端通过统一 `ApiClient` 调 Tauri invoke。
- **围棋基础链路**：棋盘状态、落子、Pass、Undo、前进/后退、跳转、编辑模式 add/remove stone、棋谱路径读取已接入。
- **SGF 基础能力**：Rust 端已有 `load_sgf` / `save_sgf`，前端已接入 Tauri dialog/fs 插件，Open/Save SGF 按钮不再是空实现。
- **引擎基础能力**：Rust/Tauri 端已有 engine1/engine2 启停、ponder、genmove、analysis event；前端已增加引擎配置入口和 Start/Stop/Ponder/Genmove 控制入口。
- **配置系统基础**：Rust 端 `AppConfig` 持久化已存在，前端已补 `getConfig/saveConfig` 类型和 client 方法。
- **首页产品化第一轮**：主页保留大棋盘布局，右侧 Engine/Winrate/Preview/Comment/MoveList 结构完成；明显未实现按钮已禁用，不再“点了没反应”。
- **设置面板第一版**：新增现代 Settings Dialog，包含 General / Engines / Board / Theme 分类，支持暗色主题、棋盘显示选项和 engine profiles 编辑。
- **验证状态**：`npm --prefix app run check`、`npm --prefix app run build`、`cargo check` 已通过；剩余 Svelte 警告主要来自既有 layout 组件初始化写法。

### 当前短板

- **引擎主链路还需要实测打通**：配置 engine command 后启动、收到 analysis、候选点/胜率图实时更新这条链路需要下一轮重点验证和修补。
- **SGF 文件流需要桌面端实测**：Open/Save 已接线，但仍需在 Tauri app 中验证权限、路径、dirty 状态、文件名状态栏表现。
- **设置项还只是骨架**：Yzy 的大量分析/显示/规则/主题选项还未抽象成可持久化配置。
- **变化树/评论仍偏只读**：MoveList 现在能导航，但 Add/Delete/Flatten 暂未实现；Comment 区还不是 SGF comment 编辑器。
- **Web/server 模式仍是占位**：`HttpClient` 只保留基础 stub，桌面端是当前优先级。

---

## 与 Kaya 的差异化定位

### Kaya 是什么

Kaya 是一个用 TypeScript + React + Tauri v2 构建的围棋应用。核心特点：
- **全 TypeScript**：围棋逻辑从 Sabaki 移植，后端是 Tauri Rust 壳
- **AI 走 ONNX 路线**：把 KataGo 模型转成 ONNX，通过 ONNX Runtime 在浏览器/WASM 中推理
- **单次推理，无 MCTS**：只做 policy/value 一次前向传播，不做蒙特卡洛搜索
- **定位是 Sabaki 升级版**：漂亮的棋盘 + 基础分析 + 棋盘识别

### Kaya 的硬伤（我们的机会）

| 问题 | 原因 | 影响 |
|------|------|------|
| **AI 极弱** | 只做单次神经网络推理，没有 MCTS 搜索 | 分析精度远不如 KataGo 原生，无法做深度分析 |
| **无法利用 GPU** | ONNX WASM 跑在 CPU 上 | 大模型推理极慢，只能用小模型(b28c512) |
| **不支持多引擎** | 架构绑定 ONNX | 无法用 LeelaZero/Zen/Pachi 等其他引擎 |
| **无法引擎对局** | 没有 GTP 进程管理 | 不能做引擎 vs 引擎 |
| **无 Flash 分析** | 没有 KataGo analysis API | 不能快速复盘整盘棋 |
| **无领地估算** | 单次推理不输出完整 ownership | 不能做形势判断 |
| **围棋逻辑是 TS 移植** | 从 Sabaki 搬的 | 复杂场景（ko/超ko/计分）不如原生实现可靠 |

### 我们的差异化：专业级深度分析工具

**Kaya = 好看的棋盘 + 轻量AI（给休闲用户）**
**LizzieYzy Next = 专业分析平台 + 完整KataGo能力（给认真学棋的用户）**

我们的核心护城河：

1. **原生 KataGo 进程**：完整 MCTS 搜索，数千次 visits，分析精度碾压 ONNX 单次推理
2. **KataGo Analysis API**：Flash 分析秒级复盘整盘棋，Kaya 做不到
3. **多引擎支持**：KataGo/LeelaZero/Zen/任何 GTP 引擎，用户自由选择
4. **引擎对局**：引擎 vs 引擎自动对弈 + ELO 统计
5. **双引擎对比**：同时跑两个引擎对比分析
6. **完整领地估算**：kata-raw-nn 输出完整 ownership + 不确定性
7. **Python 后端**：后续可做开局识别、风格分析、棋谱统计等高级功能，Kaya 的 TS 架构做不了

简单说：**Kaya 做的是"能看棋的漂亮应用"，我们做的是"能深度研究棋的专业工具"**。不在一个赛道。

---

## 技术选型决策

### 架构：Tauri 2.x + Rust 后端（双模式部署）

**核心设计原则：前端与后端通信层抽象化，同一套前端代码支持两种部署模式。**

```
模式一：桌面端（Tauri）
┌─────────────────────────────────────────┐
│              Tauri App                   │
│  ┌─────────────────────────────────────┐│
│  │   Frontend (Svelte + Canvas)        ││
│  └──────────────┬──────────────────────┘│
│                 │ Tauri invoke            │
│  ┌──────────────▼──────────────────────┐│
│  │   Rust Backend (本地进程)           ││
│  │   GTP引擎 / SGF / 棋局逻辑 / SQLite ││
│  └──────────────┬──────────────────────┘│
│                 │ stdin/stdout (GTP)      │
│  ┌──────────────▼──────────────────────┐│
│  │   KataGo / LeelaZero (本地引擎)     ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘

模式二：Web端（服务端部署）
┌─────────────────────────────────────────┐
│  浏览器                                  │
│  ┌─────────────────────────────────────┐│
│  │   Frontend (同一套 Svelte + Canvas) ││
│  └──────────────┬──────────────────────┘│
└─────────────────┼───────────────────────┘
                  │ HTTP/WebSocket
┌─────────────────▼───────────────────────┐
│  服务器 (Rust - Axum/Actix)             │
│  ┌─────────────────────────────────────┐│
│  │   同一套 Rust 核心逻辑              ││
│  │   GTP引擎 / SGF / 棋局逻辑 / SQLite ││
│  └──────────────────┬──────────────────┘│
│                     │ stdin/stdout (GTP)  │
│  ┌──────────────────▼──────────────────┐│
│  │   KataGo (服务器本地引擎)           ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

### 关键架构决策：通信层抽象

前端不直接调用 `invoke()` 或 `fetch()`，而是通过统一的 API 抽象层：

```typescript
// src/lib/api.ts — 通信层抽象
// 桌面模式：走 Tauri invoke
// Web模式：走 HTTP/WebSocket

interface ApiClient {
  getBoard(): Promise<BoardState>;
  placeMove(x: number, y: number): Promise<BoardState>;
  startEngine(config: EngineConfig): Promise<void>;
  // ...
}
```

后端 Rust 核心逻辑编译为 library crate，桌面端通过 Tauri invoke 调用，Web端通过 Axum HTTP 路由调用，**同一套业务代码**。

```
lizzie-next/
├── crates/
│   ├── core/          ← 纯逻辑 library（Board/SGF/GTP/History）
│   ├── desktop/       ← Tauri 命令层（调用 core）
│   └── server/        ← Axum HTTP 路由层（调用 core）
└── app/               ← 前端（Svelte，通过 api.ts 切换通信模式）
```

### 为什么选 Rust 原生后端

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 桌面框架 | Tauri 2.x | 包体~10MB，内存低，Rust安全 |
| 前端框架 | Svelte | 编译后体积小，围棋UI不需要复杂状态管理 |
| 棋盘渲染 | HTML5 Canvas | 19x19网格+覆盖层，比DOM高效；热力图用WebGL |
| 图表 | ECharts | 胜率图/score分布开箱即用 |
| 后端语言 | Rust (Tauri 原生) | 单进程、零IPC开销、手机端直接可用 |
| 后端通信 | Tauri invoke | 前端直接调用 Rust 函数，零延迟 |
| 数据存储 | SQLite (rusqlite) | 棋谱库、引擎配置、分析记录 |
| 包体 | ~10MB | 无需打包 Python 解释器 |

### 手机端策略

Rust 后端 + Tauri 2.0 原生支持 iOS/Android，同一套代码覆盖桌面+手机。前端代码 100% 复用，后端代码也 100% 复用。引擎通信在手机端通过远程引擎（SSH/网络 KataGo）实现。

---

## 核心逻辑移植评估

现有 Java 代码中需要移植的纯逻辑约 **3,500-4,500 行**（剥离UI耦合后）：

| 模块 | 纯逻辑行数 | 优先级 | 说明 |
|------|-----------|--------|------|
| Board 规则引擎 | ~450行 | P0 | 落子/提子/ko/计分 |
| BoardHistoryNode/List | ~550行 | P0 | 棋谱树数据结构 |
| SGFParser | ~550行 | P0 | SGF解析/写入 |
| Leelaz (GTP通信) | ~650行 | P0 | 引擎进程管理+输出解析 |
| MoveData | ~280行 | P0 | 引擎输出解析 |
| EngineManager | ~450行 | P1 | 多引擎管理/引擎对局 |
| AnalysisEngine | ~220行 | P1 | KataGo analysis模式 |
| GIBParser | ~90行 | P2 | Tygem GIB格式 |
| Config结构 | ~180行 | P1 | 配置定义+读写 |
| Stone/Zobrist/GameInfo | ~235行 | P0 | 基础数据结构 |

---

## 项目结构（Cargo Workspace）

```
lizzie-next/
├── crates/
│   ├── core/                            # 纯逻辑 library crate
│   │   ├── src/
│   │   │   ├── go/                      # 围棋核心
│   │   │   │   ├── mod.rs
│   │   │   │   ├── board.rs             # 规则引擎
│   │   │   │   ├── stone.rs             # Stone枚举
│   │   │   │   ├── board_history.rs     # 棋谱树
│   │   │   │   ├── zobrist.rs          # Zobrist哈希
│   │   │   │   └── sgf.rs              # SGF解析/写入
│   │   │   ├── engine/                  # 引擎通信
│   │   │   │   ├── mod.rs
│   │   │   │   ├── gtp.rs              # GTP协议
│   │   │   │   ├── move_data.rs        # 输出解析
│   │   │   │   ├── engine_manager.rs   # 多引擎管理
│   │   │   │   └── analysis.rs         # KataGo analysis
│   │   │   └── config.rs               # 配置管理
│   │   └── Cargo.toml
│   ├── desktop/                         # Tauri 桌面端（调用 core）
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs                   # Tauri Builder + invoke 命令
│   │   │   └── commands/               # Tauri invoke 命令
│   │   │       ├── board_cmd.rs
│   │   │       ├── engine_cmd.rs
│   │   │       └── sgf_cmd.rs
│   │   ├── capabilities/
│   │   ├── tauri.conf.json
│   │   └── Cargo.toml
│   └── server/                          # Web 服务端（调用 core）
│       ├── src/
│       │   ├── main.rs                  # Axum 服务器
│       │   └── routes/                  # HTTP/WebSocket 路由
│       │       ├── board_route.rs
│       │       ├── engine_route.rs
│       │       └── sgf_route.rs
│       └── Cargo.toml
├── app/                                 # 前端 (Svelte)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api/                    # 通信层抽象（关键！）
│   │   │   │   ├── types.ts            # 共享类型定义
│   │   │   │   ├── client.ts           # ApiClient 接口
│   │   │   │   ├── tauri-client.ts     # 桌面模式：Tauri invoke
│   │   │   │   └── http-client.ts      # Web模式：HTTP/WebSocket
│   │   │   ├── board/                  # 棋盘渲染
│   │   │   │   ├── BoardCanvas.svelte
│   │   │   │   ├── StoneRenderer.ts
│   │   │   │   ├── OverlayRenderer.ts
│   │   │   │   └── CoordinateSystem.ts
│   │   │   ├── charts/
│   │   │   │   ├── WinrateGraph.svelte
│   │   │   │   └── ScoreChart.svelte
│   │   │   ├── tree/
│   │   │   │   ├── VariationTree.svelte
│   │   │   │   └── MoveList.svelte
│   │   │   └── panels/
│   │   │       ├── EnginePanel.svelte
│   │   │       └── CommentPanel.svelte
│   │   ├── App.svelte
│   │   └── main.ts
│   ├── index.html
│   ├── package.json
│   ├── svelte.config.js
│   ├── tsconfig.json
│   └── vite.config.ts
├── Cargo.toml                           # Workspace 根
└── package.json
```

### 通信层设计（双模式切换核心）

```typescript
// api/client.ts
export interface ApiClient {
  // Board
  getBoard(): Promise<BoardState>;
  placeMove(x: number, y: number): Promise<BoardState>;
  passMove(): Promise<BoardState>;
  undoMove(): Promise<BoardState>;

  // Engine
  startEngine(config: EngineConfig): Promise<void>;
  stopEngine(): Promise<void>;
  getEngineStatus(): Promise<EngineStatus>;
  onAnalysisUpdate(callback: (data: AnalysisData) => void): void;

  // SGF
  loadSgf(path: string): Promise<BoardState>;
  saveSgf(path: string): Promise<void>;
}

// 桌面模式：零延迟
// api/tauri-client.ts
export class TauriClient implements ApiClient {
  async placeMove(x: number, y: number) {
    return invoke<BoardState>('place_move', { x, y });
  }
  onAnalysisUpdate(cb) {
    // Tauri event: listen('analysis-update', cb)
  }
}

// Web模式：HTTP + WebSocket
// api/http-client.ts
export class HttpClient implements ApiClient {
  async placeMove(x: number, y: number) {
    return fetch('/api/board/move', { method: 'POST', body: JSON.stringify({x, y}) });
  }
  onAnalysisUpdate(cb) {
    // WebSocket: ws://server/analysis
  }
}

// 自动检测运行环境
export function createClient(): ApiClient {
  if (window.__TAURI__) {
    return new TauriClient();
  }
  return new HttpClient();
}
```

---

## 近期执行计划（更新于 2026-05-16）

### P0：打通核心使用闭环

1. **引擎配置 → 启动 → 分析显示**
   - 完善 Settings / Engines：默认引擎、命令行校验、启动失败错误展示。
   - 实测 `start_engine` / `engine:analysis` / `stop_engine` 全链路。
   - 确认候选点 overlay、EnginePanel、WinrateGraph 能随分析数据同步刷新。

2. **SGF 打开/保存桌面实测**
   - 用 Tauri 桌面端验证 native open/save dialog 和 fs 权限。
   - 打开 SGF 后刷新 board/tree/statusbar；保存后清除 dirty 标记。
   - 处理取消选择、解析失败、写入失败的 UI 错误提示。

3. **首页右侧工作台二次 polish**
   - 进一步统一 Engine / Graph / Preview / Comment / MoveList 的 card header、空状态和 CTA。
   - 没配置引擎时只保留一个主入口，避免重复配置按钮。
   - 减少空框感，让首页看起来像“等待分析的工作台”而不是未完成页面。

### P1：把 Yzy 常用设置现代化

1. **Analysis Settings**
   - 候选点显示数量。
   - 显示胜率 / 计算量 / 目数开关。
   - 单步最大分析限制：时间 / visits。
   - 胜率图模式和 blunder 阈值。

2. **Board / Layout Settings**
   - 坐标、手数、候选点颜色、小棋盘、评论栏、胜率图显示开关。
   - 布局 preset：默认、紧凑、分析优先。
   - Preview 棋盘显示/隐藏和尺寸持久化。

3. **Theme Settings**
   - Modern Dark / Paper Light / Classic Yzy 预设。
   - 棋盘材质、背景、棋子资源、accent 色和胜率颜色预览。
   - 主题实时预览，但只保存真实支持的字段。

### P2：棋谱树和评论进入可编辑状态

1. **SGF Comment 编辑**
   - Comment 区改为 editable textarea。
   - 当前节点 comment 读写进入 `BoardData` / SGF。
   - 保存 SGF 时写回注释。

2. **变化树操作**
   - 后端补 branch mutation API：删除分支、提升分支、flatten、创建变化。
   - 前端 MoveList/VariationTree 接真实操作，并对删除类动作加确认。
   - 从 `treePath` 扩展为完整 tree 数据结构显示。

### P3：专业分析能力

1. **KataGo Analysis API / Flash 分析**
   - 批量分析整盘棋。
   - 进度、取消、结果回填棋谱树。
   - 关键转折点和失误分类。

2. **Ownership / Influence / Policy**
   - Ownership 图和棋盘 overlay。
   - Policy 热力图。
   - 分析不确定性展示。

### P4：Web/server 和发布

- 完成 `crates/server` Axum API。
- 实现 `HttpClient` HTTP/WebSocket 通信。
- 桌面打包、自动更新、后续移动端适配。

---

## 分阶段实施计划

### Phase 0: 项目初始化（1-2天） ✅ 完成

**目标**：搭建项目骨架，跑通 Tauri + Rust 通信

- [x] 初始化 Tauri 2.x + Svelte + TypeScript 项目
- [x] 创建 Rust 后端模块结构（go/、engine/、commands/）
- [x] 实现第一个 Tauri invoke 命令（前端调 Rust 函数）
- [x] 验证 Tauri dev 模式完整启动
- [x] 配置 Cargo.toml 依赖

**验收标准**：前端点击按钮，调用 Rust 函数，返回数据并显示

---

### Phase 1: 最小可用版本（2-3周） ✅ 完成

**目标**：能看棋、能下棋、能分析

> 以下任务精确对标旧项目 Board.java / Leelaz.java / SGFParser.java / BoardHistoryNode.java 的实际实现

#### 1.1 Rust 围棋核心逻辑（对标 Board.java + BoardHistoryNode.java + BoardData.java） ✅ 完成

- [x] `go/stone.rs` — Stone枚举
- [x] `go/zobrist.rs` — Zobrist哈希
  - 对标：`Zobrist.java` — 为每个(位置,颜色)生成随机数，位置哈希=异或所有有子位置的随机数
  - 用于 ko 检测和超 ko 判定
- [ ] `go/board.rs` — 围棋规则引擎（对标 Board.java 4500行，核心逻辑~450行）
  - ✅ 已有基础版本，需补全：
  - **Ko规则**：简单ko（对比2步前的Zobrist哈希），对标 `BoardHistoryList.violatesKoRule()`
  - **提子**：flood-fill找无气群体并移除，对标 `removeDeadChain()` + `hasLibertiesHelper()`
  - **自杀判定**：落子后己方无气则非法，对标 `isSuicidal` 逻辑
  - **Pass**：空坐标，递增手数，对标 `pass()` 方法
  - **编辑模式**：`addStone()/removeStone()` 不创建历史记录，对标 Board.java:820-858
  - **坐标系转换**：`convertCoordinatesToName()/asCoordinates()`，对标 Board.java 的字母坐标系统
  - **19x19/13x13/9x9/非正方形** 支持
  - **BoardData 快照**：对标 BoardData.java 全部字段（stones, lastMove, blackToPlay, zobrist, moveNumber, captures, comment, properties, bestMoves, winrate, scoreMean, playouts 等）
- [ ] `go/board_history.rs` — 棋谱树（对标 BoardHistoryNode.java + BoardHistoryList.java）
  - **BoardHistoryNode**：对标全部字段
    - `previous` 指向父节点
    - `variations` 子节点列表（index 0 = 主干）
    - `data: BoardData`
    - `nodeInfo / nodeInfoMain` — 引擎1分析数据
    - `nodeInfo2 / nodeInfoMain2` — 引擎2分析数据
    - `analyzed / diffAnalyzed / isBest` — 分析状态
    - `extraStones` — 编辑模式额外棋子
  - **导航方法**（对标 BoardHistoryNode 全部导航）：
    - `next()` / `previous()` / `getVariation(idx)`
    - `topOfBranch()` / `isMainTrunk()` / `isFirstChild()`
    - `firstParentWithVariations()`
  - **BoardHistoryList**：
    - `head` 指针指向当前节点
    - `place()` — 落子创建新节点或匹配已有历史（对标优化：匹配下一个历史节点时直接前进）
    - `addOrGoto()` — 已有位置则导航，否则创建分支
    - `violatesKoRule()` / `violatesSuperko()` — ko判定
    - `previous()` / `next()` / `nextVariation(idx)` / `toStart()` / `goToMoveNumber()`
- [ ] `go/sgf.rs` — SGF解析/写入（对标 SGFParser.java 3471行，核心~550行）
  - **属性解析**（对标全部已解析的 SGF 属性）：
    - 着手：`B`/`W`（落子）、`AB`/`AW`/`AE`（放置/移除棋子）
    - 游戏信息：`SZ`（棋盘大小，支持`SZ[13:19]`）、`KM`/`KO`（贴目）、`PB`/`PW`（选手）、`RE`（结果）、`HA`（让子）、`DT`/`AP`/`CA`
    - 节点：`C`（注释）、`N`（节点名）、`MN`（手数覆盖）
    - 标记：`LB`/`CR`/`SQ`/`MA`/`TR`
    - Lizzie扩展：`LZ`/`LZ2`（引擎分析数据）、`LZOP`/`LZOP2`（开局分析）、`FIT`、`DZ`、`DD`
  - **变着处理**：支持 MultiGo 格式和标准格式，对标 `isMultiGo` 检测
  - **编码**：对标 `EncodingDetector`，回退 WINDOWS-1252 → GB18030
  - **SGF 写入**：深度优先遍历变着树，转义 `]` 和 `\`
  - **边界情况**：pass空坐标、>=52路棋盘用`x_y`格式、贴目归一化（>=200则除100）

#### 1.2 Rust 引擎通信（对标 Leelaz.java 3689行 + MoveData.java + AnalysisEngine.java）

- [ ] `engine/gtp.rs` — GTP引擎通信（对标 Leelaz.java，核心~650行）
  - **进程管理**：`tokio::process::Command` 启动引擎子进程，对标 `ProcessBuilder`
  - **命令编号系统**：对标 `cmdNumber` / `currentCmdNum`，跟踪请求-响应对应
  - **命令队列**：对标 `cmdQueue` + `trySendCommandFromQueue()`，优化：移除待执行的 analyze 命令
  - **全部GTP命令**（对标 Leelaz.java 使用的全部命令）：
    - 基础：`name`/`version`/`list_commands`/`quit`/`komi`/`boardsize`/`clear_board`/`undo`/`play`
    - 分析：`lz-analyze`/`kata-analyze`/`analyze`（Sayuri）
    - 带分析落子：`lz-genmove_analyze`/`kata-genmove_analyze`/`genmove_analyze`
    - 控制：`stop`/`genmove`/`fixed_handicap`/`place_free_handicap`/`time_left`
    - KataGo特有：`kata-set-rules`/`kata-set-param`/`kata-get-param`/`kata-get-rules`/`kata-raw-nn`
    - Leela特有：`lz-setoption`/`heatmap`
  - **引擎类型检测**：对标 `isKatago`/`isSai`/`isLeela`/`isLeela0110`/`isZen`/`isSayuri`/`isGolaxy`
  - **Pondering控制**：对标 `ponder()`/`nameCmd()`/`togglePonder()`
  - **输出流解析**：后台线程读取stdout，对标 `read()` → `parseLine()` 循环
- [ ] `engine/move_data.rs` — 引擎输出解析（对标 MoveData.java，核心~280行）
  - **三种分析格式解析**：
    - `fromInfo()` — Leela 0.17+ 格式（winrate/lcb/prior 除以100）
    - `fromInfoKatago()` — KataGo 格式（winrate/lcb/prior 乘以100）+ scoreMean/scoreStdev/ownership/pvVisits/movesOwnership/isSymmetryOf
    - `fromInfoSai()` — SAI 格式 + areas/scoreLead
  - **旧格式兼容**：`fromSummary()`/`fromSummaryKata()`/`fromSummaryZen()`/`fromSummaryLeela0110()`
  - **全部字段**：coordinate, playouts, winrate, variation, pvVisits, lcb, policy, scoreMean, scoreStdev, isKataData, isSaiData, order, isNextMove, bestWinrate, bestScoreMean, isSymmetry, movesEstimateArray
- [ ] `engine/analysis.rs` — KataGo Analysis模式（对标 AnalysisEngine.java，核心~220行）
  - JSON query构建：maxVisits, includePVVisits, includeOwnership, includeMovesOwnership, initialStones, rules, komi, moves, analyzeTurns, overrideSettings
  - JSON response解析：moveInfos, ownership
  - 批量分析：遍历棋谱树发送请求

#### 1.3 Tauri 命令层

- [x] `commands/board_cmd.rs` — 棋盘状态命令
  - `get_board` — 获取当前棋盘状态 ✅ 已有
  - `place_move` — 落子 ✅ 已有
  - `pass_move` — pass ✅ 已有
  - `undo_move` — 悔棋 ✅ 已有（后续需增强为完整历史树语义）
  - `goto_move` — 跳转到指定手数 ✅ 已有
  - `next_move` / `previous_move` — 前进/后退 ✅ 已有
  - `add_stone` / `remove_stone` — 编辑模式 ✅ 已有
- [x] `commands/engine_cmd.rs` — 引擎管理命令
  - `start_engine` — 启动引擎（传入路径和参数）✅ 已有
  - `stop_engine` — 停止引擎 ✅ 已有
  - `get_engine_status` — 引擎状态 ✅ 已有
  - `get_analysis` — 获取当前分析数据（MoveData列表）✅ 已有
  - `toggle_ponder` — 开关 pondering ✅ 已有
  - `start_engine2` / `stop_engine2` / `get_analysis2` — 双引擎基础命令 ✅ 已有
- [x] `commands/sgf_cmd.rs` — SGF文件命令
  - `load_sgf` — 加载SGF ✅ 已有基础版本
  - `save_sgf` — 保存SGF ✅ 已有基础版本

#### 1.4 前端核心UI

- [x] `BoardCanvas.svelte` — 棋盘渲染（Canvas）
  - 19x19网格 + 星位
  - 黑白棋子主题资源
  - 最后一手标记
  - 鼠标悬停预览
  - 点击落子交互
  - 自适应缩放
- [x] `OverlayRenderer.ts` — 建议覆盖基础版（对标 BoardRenderer.java 的覆盖层渲染）
  - 候选着手显示
  - 胜率数字标注
  - 变着序列显示
- [x] `WinrateGraph.svelte` — 胜率图（对标 WinrateGraph.java）
  - ECharts折线图
  - 鼠标悬停显示数值
  - 点击跳转到对应着手
- [x] `EnginePanel.svelte` — 引擎信息（对标 LizzieFrame 的引擎信息区域）
  - 引擎名称/状态
  - 当前visits/playouts
  - 最佳着手列表
  - 启停/ponder/genmove 控制入口
- [x] `App.svelte` — 主布局
  - 左：棋盘
  - 右：引擎信息 + 胜率图 + Preview + Comment + MoveList
  - 键盘快捷键（方向键导航等）
  - Settings Dialog / SGF Open-Save / Window controls 接入
- [x] `api/` — Tauri invoke 封装
  - 封装所有 Rust 命令调用
  - 类型安全
  - Config get/save 已接入

**验收标准**：
1. 启动 KataGo 引擎
2. 在棋盘上落子，引擎实时分析
3. 看到候选着手和胜率
4. 打开/保存 SGF 文件（支持变着）
5. 胜率图实时更新

---

### Phase 2: 核心功能补齐（2-3周）

**目标**：达到日常使用水平

#### 2.1 棋谱树和变着

- [ ] `VariationTree.svelte` — 变着树
  - SVG渲染棋谱树
  - 当前位置高亮
  - 点击跳转
  - 分支折叠/展开
- [ ] `MoveList.svelte` — 着手列表
  - 序号 + 坐标 + 胜率
  - 点击跳转
- [ ] Rust：变着创建/删除/切换命令
  - `create_branch` — 创建分支
  - `goto_node` — 跳转到指定节点

#### 2.2 双引擎模式

- [ ] Rust：多引擎实例管理
  - `start_second_engine` — 启动第二引擎
  - `get_analysis2` — 第二引擎分析数据
- [ ] 前端：双引擎对比视图
  - 第二列建议显示
  - 对比胜率差

#### 2.3 引擎对局

- [ ] Rust：引擎对局逻辑
  - `EngineGameInfo` 配置
  - 自动genmove交替落子
  - 时间/visits限制
  - 对局结果统计
- [ ] 前端：引擎对局设置UI
  - 选择黑白引擎
  - 设置时间/visits
  - 实时观看对局

#### 2.4 Flash/Batch 分析

- [ ] Rust：KataGo Analysis API
  - JSON query构建
  - 并行分析调度
  - 结果回填到棋谱树
- [ ] 前端：分析设置+进度UI
  - 选择分析范围
  - 进度条
  - 结果展示

#### 2.5 配置系统 ✅ 基础完成，继续扩展

- [x] Rust：配置管理
  - JSON配置文件读写
  - 引擎配置（名称、命令行、初始命令、分析间隔）
  - UI偏好基础字段
- [x] 前端：设置页面第一版
  - 引擎配置面板
  - Board/Theme/General 基础偏好面板
- [ ] 后续扩展
  - Analysis settings（候选点数量、显示胜率/计算量/目数、分析限制）
  - Layout settings（小棋盘、评论栏、胜率图、面板布局 preset）
  - Rules settings（KataGo 规则预设和高级规则字段）
  - Theme settings（主题预设、棋盘/棋子资源、颜色预览）

**验收标准**：
1. 完整的棋谱树导航和变着
2. 双引擎同时分析对比
3. 引擎对局功能
4. Flash分析整盘棋
5. 配置持久化

---

### Phase 3: 功能完善 + 体验升级（2-3周）

**目标**：超越现有 LizzieYzy 的体验

#### 3.1 视觉升级

- [ ] 着手质量评级标签（A/B/C/D/F）
- [ ] 建议着手渐变动画（不再瞬间切换）
- [ ] 领地估算颜色渐变动画
- [ ] 深色模式
- [ ] 多主题支持（从theme.txt迁移）
- [ ] 热力图/Policy可视化（WebGL）

#### 3.2 信息增强

- [ ] AI 复盘摘要面板
  - 全局胜率波动概览
  - 失误分类统计（好手/缓手/恶手占比）
  - 关键转折点自动标注
- [ ] Score 概率分布图（ECharts直方图）
- [ ] Ownership 不确定性可视化（ownershipStdev）
- [ ] 每手胜率变化量标注（Δ%）

#### 3.3 现有功能迁移

- [ ] 领地估算（kata-raw-nn）
- [ ] 死活题模式（Tsumego frame）
- [ ] 鹰眼分析（Hawk Eye）
- [ ] 右键菜单
- [ ] GTP控制台
- [ ] 棋盘同步（长期，依赖外部工具）

#### 3.4 交互优化

- [ ] 可配置快捷键
- [ ] 拖拽SGF文件打开
- [ ] 撤销/重做 (Ctrl+Z/Ctrl+Y)
- [ ] 棋盘坐标显示切换
- [ ] 着手编号显示切换

**验收标准**：
1. 视觉体验明显优于旧版
2. AI复盘功能可用
3. 鹰眼/死活题/领地估算功能齐全

---

### Phase 4: 数据管理 + 高级功能（2-3周）

#### 4.1 棋谱库

- [ ] SQLite 棋谱存储
  - 导入/导出SGF
  - 按选手/赛事/日期/标签归档
  - 全文搜索（注释内容）
- [ ] 棋谱库浏览UI

#### 4.2 KataGo 完整能力

- [ ] `shorttermWinrateLoss` 不确定性
- [ ] `allowMoves`/`avoidMoves` 限制分析范围
- [ ] `includePolicy` 每点先验概率
- [ ] `maxTime` 时间限制分析
- [ ] `includeOwnershipStdev` 领地不确定性

#### 4.3 远程引擎

- [ ] Rust：SSH远程引擎连接（ssh2 crate）
- [ ] 远程引擎配置UI

#### 4.4 多语言

- [ ] i18n框架（svelte-i18n）
- [ ] 中文/英文/日文/韩文翻译
- [ ] 从现有 DisplayStrings.properties 迁移

**验收标准**：
1. 棋谱库可以管理1000+棋谱
2. KataGo高级分析字段全部可用
3. 支持SSH远程引擎

---

### Phase 5: 打包发布 + 服务端部署（2-3周）

#### 5.1 桌面打包

- [ ] Windows 安装包（MSI/NSIS）
- [ ] macOS DMG
- [ ] Linux AppImage/deb
- [ ] 自动更新机制（Tauri updater）

#### 5.2 服务端部署（Web模式）

- [ ] `crates/server/` — Axum HTTP/WebSocket 服务
  - 复用 `crates/core/` 全部逻辑
  - REST API 路由（对标 Tauri invoke 命令）
  - WebSocket 实时推送分析数据
  - 多用户会话管理（每个用户独立的 Board + Engine 实例）
- [ ] 前端 `api/http-client.ts` 实现
- [ ] Docker 镜像（Rust + KataGo 一键部署）
- [ ] 用户认证基础框架

#### 5.3 手机端

- [ ] Tauri 2.0 iOS/Android 构建
- [ ] 前端响应式布局适配
- [ ] 触摸手势支持（捏合缩放、滑动导航）
- [ ] 移动端通过服务端 Web 模式访问（浏览器直接打开）

---

## 时间线总览

```
Phase 0  ██████                                              1-2天
Phase 1  ████████████████████████████                        2-3周    ← 最小可用
Phase 2  ████████████████████████████                        2-3周    ← 日常可用
Phase 3  ████████████████████████████                        2-3周    ← 超越旧版
Phase 4  ████████████████████████████                        2-3周    ← 高级功能
Phase 5  ████████████                                        1-2周    ← 发布
                                                                总计: ~10-14周
```

## 参考项目

| 项目 | 用途 | 地址 |
|------|------|------|
| kaya-go/kaya | Tauri围棋应用，UI参考（但AI路线不同） | github.com/kaya-go/kaya |
| Sabaki | 棋盘UI交互参考，围棋逻辑TS实现 | github.com/SabakiHQ/Sabaki |

## 竞品定位矩阵

```
                    轻量/休闲 ←────────────────────→ 专业/深度
                         │                              │
    Kaya (ONNX)  ────────┤                              │
    Sabaki       ────────┤                              │
    KaTrain      ───────────────────────────────────────┤
    LizzieYzy(旧)───────────────────────────────────────┤
    LizzieYzy Next ─────────────────────────────────────┤  ← 目标
                         │                              │
                    好看但AI弱                    AI强+功能全
```

我们的目标：在 KaTrain/LizzieYzy 的专业深度之上，做到 Kaya 级别的现代 UI，加上服务端部署能力让用户浏览器直接用。

## 产品力差异化清单

与旧版 LizzieYzy 相比，不只是技术栈升级，产品层面必须有明显进步：

| 维度 | 旧版 (Swing) | 新版目标 |
|------|-------------|---------|
| UI美观 | Swing BufferedImage 手绘 | Canvas 径向渐变棋子 + 动画 + 深色模式 |
| 交互流畅度 | 全量重绘、无动画 | 脏区域渲染、着子动画、建议渐变 |
| 信息密度 | 全堆在一个面板 | 分区面板，可折叠，信息层次清晰 |
| AI复盘 | 只有鹰眼逐手 | AI复盘摘要（全局报告+关键转折+失误分类） |
| 着手评级 | 只有颜色条 | A/B/C/D/F 标签 + Δ% 显示 |
| 部署方式 | 只能桌面 | 桌面 + 浏览器（服务端部署） |
| 手机端 | 无 | Tauri 原生 或 浏览器访问服务端 |
| 棋谱管理 | 零散文件 | SQLite 棋谱库 + 标签 + 搜索 |
| 远程分析 | SSH手动配置 | 一键连接云端 KataGo（服务端模式） |
| 快捷键 | 硬编码 | 可自定义 |
