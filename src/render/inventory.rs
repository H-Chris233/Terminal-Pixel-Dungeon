//!    物品栏渲染器
//!
//!    渲染玩家的背包和装备栏。
//!    直接从    ECS    World    读取    Player    的    Inventory    组件。

use crate::ecs::{ECSItem, Inventory, ItemSlot, Player};
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

        // 主布局：上部内容 + 底部提示
        let main_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Min(10),    // 主内容区
                Constraint::Length(3),  // 底部提示
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

        // 渲染装备栏
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
                Line::from(Span::styled("背包空空如也", Style::default().fg(Color::DarkGray))),
            ])
            .alignment(Alignment::Center);
            frame.render_widget(empty_text, inner_area);
        } else {
            self.render_items(frame, inner_area, &inventory.items);
        }

        // 渲染底部提示
        let hints = Paragraph::new("按数字键使用物品 | D: 丢弃 | E: 装备 | Esc: 关闭")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(80, 80, 80))),
            )
            .alignment(Alignment::Center);
        frame.render_widget(hints, main_chunks[1]);
    }

    ///    渲染装备栏
    fn render_equipment(&self, frame: &mut Frame, area: Rect, _world: &World) {
        let block = Block::default()
            .title("═══ ⚔️ 装备 ═══")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // 装备槽位
        let equipment_slots = vec![
            ("武器", "⚔️", Color::Red),
            ("头盔", "🪖", Color::LightBlue),
            ("护甲", "🛡️", Color::Blue),
            ("戒指", "💍", Color::Magenta),
            ("饰品", "📿", Color::Cyan),
        ];

        let equipment_lines: Vec<Line> = equipment_slots
            .iter()
            .map(|(slot, icon, color)| {
                Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(*color)),
                    Span::styled(
                        format!("{}: ", slot),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled("空", Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();

        let equipment_paragraph = Paragraph::new(equipment_lines);
        frame.render_widget(equipment_paragraph, inner_area);
    }

    ///    获取玩家的物品栏
    fn get_player_inventory(&self, world: &World) -> Option<Inventory> {
        for (_, (inventory, _player)) in world.query::<(&Inventory, &Player)>().iter() {
            return Some(inventory.clone());
        }
        None
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
                    Span::styled(
                        format!("{} ", icon),
                        Style::default().fg(color),
                    ),
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
            ItemType::Consumable { .. } => "🧪",
            ItemType::Throwable { .. } => "🎯",
            ItemType::Key => "🔑",
            ItemType::Quest => "📜",
        }
    }
}
