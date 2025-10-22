# Terminal Pixel Dungeon UI 系统指南

这个文档介绍了Terminal Pixel Dungeon项目中新增的UI和用户交互组件。

## 🎯 功能概览

### 完善的UI组件
- **消息系统**：彩色编码的游戏消息，支持分类和历史记录
- **对话框系统**：各种类型的对话框（确认、输入、选择等）
- **动画系统**：流畅的UI动画效果（淡入淡出、脉冲、震动等）
- **帮助系统**：完整的游戏帮助和按键说明
- **增强输入**：按键组合、序列、双击检测等

### 立即可用的特性
- ✅ 完整的控制说明和按键映射
- ✅ 实时消息反馈系统  
- ✅ 友好的用户界面
- ✅ 响应式布局适配
- ✅ 错误处理和用户提示

## 🚀 快速开始

### 运行主游戏
```bash
cargo run --release
```

### 运行UI演示
```bash
cargo run --example ui_demo
```

### 运行所有测试
```bash
cargo test --workspace
```

## 🎮 游戏控制

### 基础移动
- `h/j/k/l` - vi风格方向键（左/下/上/右）
- `y/u/b/n` - 对角线移动
- `Arrow Keys` - 方向键移动
- `w/a/s` - 部分WASD支持

### 游戏动作
- `.` - 等待/跳过回合
- `g` - 拾取物品
- `d` - 丢弃物品
- `>` - 下楼梯
- `<` - 上楼梯

### 界面控制
- `i` - 打开物品栏
- `c` - 角色信息
- `?` - 帮助界面
- `m` - 消息历史
- `ESC` - 暂停/返回
- `q` - 退出游戏

### 快捷操作
- `1-9` - 使用快捷栏物品
- `SHIFT + 方向键` - 攻击指定方向

## 🔧 UI 组件使用

### 消息系统

```rust
use ui::{GameMessage, MessageType};

// 创建不同类型的消息
let info_msg = GameMessage::info("这是一条信息".to_string());
let combat_msg = GameMessage::combat("你对敌人造成了15点伤害！".to_string());
let warning_msg = GameMessage::warning("小心前方的陷阱".to_string());

// 添加到UI系统
ui.add_message(info_msg);
```

### 对话框系统

```rust
use ui::{DialogType, DialogManager};

let mut dialog_manager = DialogManager::new();

// 确认对话框
dialog_manager.show_dialog(DialogType::Confirm {
    message: "确定要退出游戏吗？".to_string(),
    default_yes: false,
});

// 输入对话框
dialog_manager.show_dialog(DialogType::Input {
    prompt: "请输入角色名称：".to_string(),
    current_input: String::new(),
    max_length: 16,
});

// 物品选择对话框
dialog_manager.show_dialog(DialogType::ItemSelect {
    title: "选择要使用的物品".to_string(),
    items: vec![/* 物品列表 */],
    selected_index: 0,
});
```

### 动画系统

```rust
use ui::{Animation, AnimationType, EaseType, AnimationManager};
use std::time::Duration;

let mut animation_manager = AnimationManager::new();

// 添加淡入动画
animation_manager.add_animation(
    "menu_fade_in".to_string(),
    Animation::new(
        AnimationType::FadeIn,
        Duration::from_millis(300),
        EaseType::EaseOut,
    )
);

// 添加脉冲动画
animation_manager.add_animation(
    "button_pulse".to_string(),
    Animation::infinite(
        AnimationType::Pulse,
        Duration::from_millis(1000),
        EaseType::EaseInOut,
    )
);

// 获取动画值并应用到渲染
if let Some(anim_value) = animation_manager.get_value("menu_fade_in") {
    // 使用 anim_value.alpha, anim_value.scale_x 等
}
```

### 帮助系统

```rust
use ui::HelpState;

let mut help_state = HelpState::new();

// 在游戏循环中处理帮助输入
if let Some(ref mut help) = help_state {
    if !help.handle_input(key_event) {
        // 用户关闭了帮助界面
        help_state = None;
    }
}

// 渲染帮助界面
if let Some(ref mut help) = help_state {
    help.render(f, area);
}
```

### 增强输入系统

```rust
use ui::{InputContextManager, InputMode};

let mut input_manager = InputContextManager::new();

// 切换输入模式
input_manager.push_context(InputMode::Menu);  // 进入菜单模式
input_manager.pop_context();                  // 返回上一个模式

// 处理输入事件
let events = input_manager.process_input(key_event);
for event in events {
    match event {
        EnhancedInputEvent::KeyPress(key) => { /* 处理按键 */ }
        EnhancedInputEvent::KeyHold(key, duration) => { /* 处理长按 */ }
        EnhancedInputEvent::DoubleClick(key) => { /* 处理双击 */ }
        _ => {}
    }
}
```

## 🎨 样式和主题

### 颜色编码系统
- **信息消息** - 白色
- **成功消息** - 绿色  
- **警告消息** - 黄色
- **错误消息** - 红色
- **战斗消息** - 亮红色
- **移动消息** - 亮蓝色
- **物品消息** - 青色
- **地牢消息** - 亮紫色
- **状态消息** - 橙色

### 动画类型
- `FadeIn/FadeOut` - 透明度变化
- `SlideUp/Down/Left/Right` - 滑动效果
- `Pulse` - 脉冲呼吸效果
- `Flash` - 闪烁效果
- `Shake` - 震动效果
- `Bounce` - 弹跳效果
- `Grow/Shrink` - 缩放效果
- `Typewriter` - 打字机效果

## 🔍 帮助系统内容

帮助系统包含以下主题：
- **Controls** - 完整的控制说明
- **Combat** - 战斗机制解释
- **Items** - 物品系统介绍
- **Dungeon** - 地牢探索指南
- **Character** - 角色系统说明
- **Tips & Tricks** - 游戏技巧
- **About** - 关于游戏

### 帮助界面操作
- `Tab/Shift+Tab` - 切换主题
- `↑/↓` - 浏览条目
- `←/→` - 切换主题（替代）
- `/` - 搜索帮助内容
- `ESC` - 关闭帮助

## 🧪 测试和演示

### 运行UI演示
```bash
cargo run --example ui_demo
```

演示包含：
1. **消息系统演示** - 各种消息类型
2. **对话框演示** - 不同对话框样式
3. **动画演示** - UI动画效果
4. **帮助演示** - 帮助系统功能
5. **输入演示** - 增强输入处理

### 单元测试
```bash
# 测试UI组件
cargo test -p ui

# 测试所有模块
cargo test --workspace

# 显示测试输出
cargo test -- --nocapture
```

## 📁 代码结构

```
src/ui/src/
├── input/
│   ├── enhanced_input.rs    # 增强输入处理
│   ├── actions.rs           # 动作映射
│   ├── navigation.rs        # 导航控制
│   └── mod.rs
├── render/
│   ├── animation.rs         # 动画系统
│   ├── dialogs.rs          # 对话框组件
│   ├── messages.rs         # 消息系统
│   ├── dungeon.rs          # 地牢渲染
│   ├── hud.rs             # HUD渲染
│   ├── inventory.rs        # 物品栏渲染
│   └── mod.rs
├── states/
│   ├── help.rs             # 帮助系统
│   ├── menu.rs             # 菜单状态
│   ├── game.rs             # 游戏状态
│   └── mod.rs
└── lib.rs                  # UI库入口
```

## 🔮 高级功能

### 自定义按键映射
```rust
let mut key_mapping = KeyMapping::new();
key_mapping.set_mapping(
    "ctrl+s".to_string(),
    "save_game".to_string(),
    Some(InputMode::Game)
);
```

### 消息过滤和搜索
```rust
// 搜索帮助内容
let results = help_database.search("combat");

// 过滤消息类型
let combat_messages: Vec<_> = message_system
    .get_all_messages()
    .iter()
    .filter(|msg| matches!(msg.msg_type, MessageType::Combat))
    .collect();
```

### 动画组合
```rust
// 创建复合动画效果
animation_manager.add_animation("fade_in".to_string(), fade_animation);
animation_manager.add_animation("slide_up".to_string(), slide_animation);

// 动画完成后自动清理
animation_manager.update(); // 在主循环中调用
```

## 🐛 故障排除

### 常见问题

**Q: 终端不支持颜色显示怎么办？**
A: 系统会自动降级到单色模式，功能不受影响。

**Q: 输入响应延迟？**  
A: 检查终端设置，确保raw mode正确启用。

**Q: 动画卡顿？**
A: 可以通过`animation_manager.set_global_speed()`调节动画速度。

**Q: 帮助文本显示不完整？**
A: 调整终端窗口大小，或使用滚动查看完整内容。

### 性能优化

```rust
// 减少动画数量
animation_manager.clear();

// 限制消息历史
message_system.set_max_messages(50);

// 使用缓存渲染
render_system.enable_cache(true);
```

## 📈 扩展开发

### 添加新的消息类型
```rust
// 在 MessageType 枚举中添加新类型
pub enum MessageType {
    // ... 现有类型
    Magic,     // 新增：魔法消息
}

impl MessageType {
    pub fn color(&self) -> Color {
        match self {
            // ... 现有映射
            MessageType::Magic => Color::Magenta,
        }
    }
}
```

### 创建自定义对话框
```rust
// 实现新的对话框类型
pub enum DialogType {
    // ... 现有类型  
    ColorPicker { current_color: Color },
}

// 在 DialogState::handle_input 中处理
```

### 添加新动画效果
```rust
pub enum AnimationType {
    // ... 现有类型
    Rotate,    // 新增：旋转动画
    Scale,     // 新增：缩放动画
}
```

## 📝 总结

新的UI系统为Terminal Pixel Dungeon提供了：

1. **完整的用户体验** - 从消息反馈到帮助系统
2. **现代化的交互** - 动画、对话框、增强输入
3. **可扩展的架构** - 模块化设计，便于添加新功能
4. **立即可用** - 开箱即用，无需额外配置

所有组件都经过充分测试，并提供了详细的使用示例。游戏现在具备了专业roguelike游戏应有的所有UI特性！

---
*更多详细信息请查看源代码中的文档注释和示例。*