# AGENTS.md

Rust 终端文字 RPG 引擎。二进制 crate（非 lib），入口 `src/main.rs`。

## 命令

```bash
cargo run          # 必须在仓库根目录：相对路径加载 data.toml
cargo check
cargo test         # 目前无测试
cargo fmt -- --check
```

无 CI / 无 README。`edition = "2024"`。依赖：`serde` + `toml` + `rustyline`（Tab 补全与行编辑）。

## 架构（别猜错）

| 模块 | 职责 |
|------|------|
| `data.toml` | **唯一**游戏内容与 UI 文案源 |
| `data.rs` | TOML → `Raw*` → `build_*` 转运行时类型 |
| `command.rs` | 输入解析 → `Command` 枚举 |
| `completion.rs` | rustyline `Completer`：命令与上下文参数 |
| `game.rs` | 主循环、命令处理、对话状态机 |
| `player.rs` / `shop.rs` / `container.rs` / `item/` | 领域逻辑 |

流程：`GameData::load("data.toml")` → `Game::new` → `Game::run`。

- **UI 文案**：走 `data.msg` / `data.err` 或 `data.raw.*_ui` 的 `{placeholder}` 替换，不要在 `game.rs` 硬编码中文提示（`error.rs` 的 `Display` 是兜底，用户可见路径优先用 data）。
- **物品**：`ItemDef`（定义，注册表） vs `ItemInstance`（实例 ID + 耐久）。匹配多用**显示名**子串（忽略大小写），商店/容器/装备同理。
- **背包**：`equip` **不能**装背包槽；必须用 `swapbackpack` / `swapbp`。溢出物品进 `player.loose_items`。
- **世界容器**：`open` / `take` / `put` / `contents` 用 **容器 id**（如 `chest_1`），不是显示名「破旧木箱」。
- **对话中**：`active_dialogue.is_some()` 时走数字选项 / `q`，不走 `Command::parse`。
- **商店**：`Game::new` 只取 `data.raw.shops` 的**第一家**。
- **套装**：`SetEffect` / `ItemDef.set_id` 已有结构，但 `data.toml` 未加载套装，`set_effects` 始终为空。

## 改数据 vs 改代码

- 新物品 / NPC / 对话 / 商店库存 / 文案 → 优先改 `data.toml`，并保证 `data.rs` 的字符串匹配分支覆盖（`item_type`、`rarity`、`equip_slot`、`effect_type`）。
- 新命令：`Command` 枚举 + `parse` + `game::handle_command` + help 行（`data.toml` `[help]`）。

## 注意

- 运行目录必须能读到 `./data.toml`。
- 当前源码未完全 `rustfmt`；改文件后可 `cargo fmt`，勿为「对齐格式」做无关大改。
- 无测试；验证以 `cargo check` + 手动 `cargo run` 交互为主。
