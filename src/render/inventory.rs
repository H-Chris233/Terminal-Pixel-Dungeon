//!    物品栏渲染器
//!
//!    渲染玩家的背包和装备栏，显示物品描述和真实装备数据。

use crate::ecs::{ECSItem, EquippedItems, Inventory, ItemSlot, Player};
use hecs::World;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

///    物品栏渲染器
pub struct InventoryRenderer;

impl InventoryRenderer {
    pub fn new() -> Self {
        Self
    }

    ///    渲染物品栏
    pub fn render(&self, frame: &mut Frame, area: Rect, world: &World) {
        //    获取玩家物品栏
        let inventory = self.get_player_inventory(world);

        if inventory.is_none() {
            let text = Paragraph::new("📦 未找到物品栏数据")
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .title("═══ 物品栏 ═══")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Double)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let inventory = inventory.unwrap();

        // 主布局：上部内容 + 底部描述 + 底部提示
        let main_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // 主内容区（装备栏 + 物品栏）
                Constraint::Length(4), // 物品描述区
                Constraint::Length(3), // 底部提示
            ])
            .split(area);

        // 分割区域：左边装备栏，右边物品栏
        let main_layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 装备栏
                Constraint::Percentage(70), // 物品栏
            ])
            .split(main_chunks[0]);

        // 渲染装备栏（读取真实装备数据）
        self.render_equipment(frame, main_layout[0], world);

        // 渲染物品栏
        let block = Block::default()
            .title(format!(
                "═══ 📦 背包 ({}/{}) ═══",
                inventory.items.len(),
                inventory.max_slots
            ))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(main_layout[1]);
        frame.render_widget(block, main_layout[1]);

        //    渲染物品列表
        if inventory.items.is_empty() {
            let empty_text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled("🎒", Style::default().fg(Color::Gray))),
                Line::from(""),
                Line::from(Span::styled(
                    "背包空空如也",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .alignment(Alignment::Center);
            frame.render_widget(empty_text, inner_area);
        } else {
            self.render_items(frame, inner_area, &inventory.items);
        }

        // 渲染选中物品的描述（默认显示第一个物品的描述）
        let desc_text = if let Some(first_slot) = inventory.items.first() {
            if let Some(item) = &first_slot.item {
                self.get_item_description(item)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let desc_paragraph = Paragraph::new(desc_text)
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
            .block(
                Block::default()
                    .title("📖 物品描述")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(100, 100, 100))),
            );
        frame.render_widget(desc_paragraph, main_chunks[1]);

        // 渲染底部提示
        let hints = Paragraph::new(
            "数字键: 使用 | E: 装备 | Shift+E: 卸下 | Del: 丢弃 | Esc: 关闭",
        )
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(80, 80, 80))),
        )
        .alignment(Alignment::Center);
        frame.render_widget(hints, main_chunks[2]);
    }

    ///    获取物品描述文字
    fn get_item_description(&self, item: &ECSItem) -> String {
        // 尝试从 items::Item 获取完整描述
        if let Ok(ref_item) = item.to_items_item() {
            if !ref_item.description.is_empty() {
                return format!("{} — {}", item.name, ref_item.description);
            }
        }

        // 根据类型生成默认描述
        use crate::ecs::ItemType;
        match &item.item_type {
            ItemType::Wand { charges, max_charges, .. } => {
                format!("🔮 充能: {}/{}", charges, max_charges)
            }
            ItemType::Weapon { damage } => {
                format!("⚔️ 攻击力: +{}", damage)
            }
            ItemType::Armor { defense } => {
                format!("🛡️ 防御力: +{}", defense)
            }
            ItemType::Consumable { effect } => {
                use crate::ecs::ConsumableEffect;
                match effect {
                    ConsumableEffect::Healing { amount } => {
                        format!("💚 恢复 {} 点生命值", amount)
                    }
                    ConsumableEffect::Damage { amount } => {
                        format!("💥 造成 {} 点伤害", amount)
                    }
                    ConsumableEffect::Buff { stat, value, duration } => {
                        let stat_name = match stat {
                            crate::ecs::StatType::Hp => "HP上限",
                            crate::ecs::StatType::Attack => "攻击",
                            crate::ecs::StatType::Defense => "防御",
                            crate::ecs::StatType::Accuracy => "命中",
                            crate::ecs::StatType::Evasion => "闪避",
                        };
                        format!("✨ {}点{}，持续{}回合", value, stat_name, duration)
                    }
                    ConsumableEffect::Teleport => "🌀 随机传送".to_string(),
                    ConsumableEffect::Identify => "🔍 鉴定物品".to_string(),
                    ConsumableEffect::Upgrade => "⬆️ 强化已装备的武器或护甲".to_string(),
                    ConsumableEffect::RemoveCurse => "✨ 解除装备上的诅咒".to_string(),
                    ConsumableEffect::MagicMapping => "🗺️ 揭示当前层全部地图".to_string(),
                    ConsumableEffect::Experience(_) => "📈 获得大量经验值".to_string(),
                    ConsumableEffect::Invisibility => "👻 暂时隐身".to_string(),
                    ConsumableEffect::Haste => "⚡ 暂时加速".to_string(),
                    ConsumableEffect::Strength => "💪 永久提升力量".to_string(),
                    ConsumableEffect::MindVision => "👁️ 查看周围敌人位置".to_string(),
                    ConsumableEffect::Levitation => "🕊️ 暂时漂浮（无视地形）".to_string(),
                    ConsumableEffect::Purity => "🧹 解除所有负面状态".to_string(),
                    ConsumableEffect::Frost => "❄️ 冰冻敌人".to_string(),
                    ConsumableEffect::LiquidFlame => "🔥 火焰伤害".to_string(),
                    ConsumableEffect::ToxicGas => "☠️ 毒气伤害".to_string(),
                    ConsumableEffect::ParalyticGas => "⚡ 麻痹气体".to_string(),
                }
            }
            ItemType::Ring { defense_bonus, crit_bonus } => {
                format!("💍 防御+{} 暴击+{}%", defense_bonus, (crit_bonus * 100.0) as u32)
            }
            ItemType::Throwable { damage, range } => {
                format!("🎯 伤害: {}-{} (射程:{})", damage.0, damage.1, range)
            }
            ItemType::Key => "🔑 用于开启锁定的门或宝箱".to_string(),
            ItemType::Quest => "📜 任务物品".to_string(),
        }
    }

    ///    渲染装备栏（读取真实 EquippedItems 数据）
    fn render_equipment(&self, frame: &mut Frame, area: Rect, world: &World) {
        let block = Block::default()
            .title("═══ ⚔️ 装备 ═══")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // 读取真实装备数据（克隆避免借用问题）
        let equipped_data: Option<(Option<ECSItem>, Option<ECSItem>, [Option<ECSItem>; 2])> = world
            .query::<(&EquippedItems, &Player)>()
            .iter()
            .next()
            .map(|(_, (eq, _))| (eq.weapon.clone(), eq.armor.clone(), eq.rings.clone()));

        let (weapon_name, has_weapon) = equipped_data
            .as_ref()
            .and_then(|(w, _, _)| w.as_ref())
            .map(|w| (w.name.as_str(), true))
            .unwrap_or(("空", false));
        let weapon_color = if has_weapon { Color::Yellow } else { Color::DarkGray };

        let (armor_name, has_armor) = equipped_data
            .as_ref()
            .and_then(|(_, a, _)| a.as_ref())
            .map(|a| (a.name.as_str(), true))
            .unwrap_or(("空", false));
        let armor_color = if has_armor { Color::Yellow } else { Color::DarkGray };

        let ring1_name = equipped_data
            .as_ref()
            .and_then(|(_, _, r)| r[0].as_ref())
            .map(|r| r.name.as_str())
            .unwrap_or("空");
        let ring1_color = if ring1_name != "空" { Color::Magenta } else { Color::DarkGray };
        let ring2_name = equipped_data
            .as_ref()
            .and_then(|(_, _, r)| r[1].as_ref())
            .map(|r| r.name.as_str())
            .unwrap_or("空");
        let ring2_color = if ring2_name != "空" { Color::Magenta } else { Color::DarkGray };

        let equipment_lines = vec![
            Line::from(vec![
                Span::styled("⚔️ ", Style::default().fg(Color::Red)),
                Span::styled("武器: ", Style::default().fg(Color::Gray)),
                Span::styled(weapon_name, Style::default().fg(weapon_color)),
            ]),
            Line::from(vec![
                Span::styled("🛡️ ", Style::default().fg(Color::Blue)),
                Span::styled("护甲: ", Style::default().fg(Color::Gray)),
                Span::styled(armor_name, Style::default().fg(armor_color)),
            ]),
            Line::from(vec![
                Span::styled("💍 ", Style::default().fg(Color::Magenta)),
                Span::styled("戒指1: ", Style::default().fg(Color::Gray)),
                Span::styled(ring1_name, Style::default().fg(ring1_color)),
            ]),
            Line::from(vec![
                Span::styled("💍 ", Style::default().fg(Color::Magenta)),
                Span::styled("戒指2: ", Style::default().fg(Color::Gray)),
                Span::styled(ring2_name, Style::default().fg(ring2_color)),
            ]),
        ];

        let equipment_paragraph = Paragraph::new(equipment_lines);
        frame.render_widget(equipment_paragraph, inner_area);
    }

    ///    获取玩家的物品栏
    fn get_player_inventory(&self, world: &World) -> Option<Inventory> {
        world
            .query::<(&Inventory, &Player)>()
            .iter()
            .next()
            .map(|(_, (inventory, _player))| inventory.clone())
    }

    ///    渲染物品列表
    fn render_items(&self, frame: &mut Frame, area: Rect, items: &[ItemSlot]) {
        let item_lines: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                let (name, color, quantity, icon) = match &slot.item {
                    None => ("空".to_string(), Color::DarkGray, 1, "□"),
                    Some(item) => {
                        let color = self.get_item_color(item);
                        let quantity = item.quantity;
                        let icon = self.get_item_icon(item);
                        (item.name.clone(), color, quantity, icon)
                    }
                };

                let quantity_str = if quantity > 1 {
                    format!(" x{}", quantity)
                } else {
                    String::new()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", index + 1),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(
                        name,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(quantity_str, Style::default().fg(Color::Rgb(120, 120, 120))),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(item_lines);
        frame.render_widget(list, area);
    }

    ///    根据物品类型获取颜色
    fn get_item_color(&self, item: &ECSItem) -> Color {
        use crate::ecs::ItemType;
        use items::ItemTrait;

        if let Ok(reference_item) = item.to_items_item() {
            return reference_item.rarity().color();
        }

        match &item.item_type {
            ItemType::Weapon { .. } => Color::Red,
            ItemType::Armor { .. } => Color::Blue,
            ItemType::Wand { .. } => Color::Cyan,
            ItemType::Ring { .. } => Color::Magenta,
            ItemType::Consumable { .. } => Color::Green,
            ItemType::Throwable { .. } => Color::LightMagenta,
            ItemType::Key => Color::LightYellow,
            ItemType::Quest => Color::Magenta,
        }
    }

    /// 根据物品类型获取图标
    fn get_item_icon(&self, item: &ECSItem) -> &str {
        use crate::ecs::ItemType;

        match &item.item_type {
            ItemType::Weapon { .. } => "⚔️",
            ItemType::Armor { .. } => "🛡️",
            ItemType::Wand { .. } => "🔮",
            ItemType::Ring { .. } => "💍",
            ItemType::Consumable { .. } => "🧪",
            ItemType::Throwable { .. } => "🎯",
            ItemType::Key => "🔑",
            ItemType::Quest => "📜",
        }
    }
}
