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
            let text = Paragraph::new("No    inventory    data")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let inventory = inventory.unwrap();

        //    创建边框
        let block = Block::default()
            .title(format!(
                "Inventory    ({}/{})",
                inventory.items.len(),
                inventory.max_slots
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        //    渲染物品列表
        if inventory.items.is_empty() {
            let empty_text = Paragraph::new("Empty")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty_text, inner_area);
        } else {
            self.render_items(frame, inner_area, &inventory.items);
        }
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
                let (name, color, quantity) = match &slot.item {
                    None => ("Empty".to_string(), Color::DarkGray, 1),
                    Some(item) => {
                        let color = self.get_item_color(item);
                        let quantity = item.quantity;
                        (item.name.clone(), color, quantity)
                    }
                };

                let quantity_str = if quantity > 1 {
                    format!("    x{}", quantity)
                } else {
                    String::new()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{}:    ", index + 1),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        name,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(quantity_str, Style::default().fg(Color::DarkGray)),
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

        match &item.item_type {
            ItemType::Weapon { .. } => Color::Red,
            ItemType::Armor { .. } => Color::Blue,
            ItemType::Consumable { .. } => Color::Green,
            ItemType::Key => Color::LightYellow,
            ItemType::Quest => Color::Magenta,
        }
    }
}
