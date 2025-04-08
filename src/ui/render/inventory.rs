//src/ui/render/inventory.rs
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{
    hero::Hero,
    items::{Item, ItemType},
};

/// 物品栏渲染器（支持分页和动态高亮）
pub struct InventoryRenderer {
    pub page: usize,               // 当前页码
    pub selected_index: usize,     // 选中项索引
    pub scroll_offset: usize,       // 滚动偏移
    pub max_items_per_page: usize,  // 每页最大显示数
}

impl InventoryRenderer {
    pub fn new() -> Self {
        Self {
            page: 0,
            selected_index: 0,
            scroll_offset: 0,
            max_items_per_page: 10, // 经典像素地牢每页显示10个物品
        }
    }

    /// 主渲染方法（整合分页和选择高亮）
    pub fn render<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Rect,
        hero: &Hero,
    ) {
        // 1. 计算分页数据
        let items = &hero.inventory.items;
        let total_pages = (items.len() + self.max_items_per_page - 1) / self.max_items_per_page;
        self.page = self.page.min(total_pages.saturating_sub(1));
        
        // 2. 创建带边框的区块
        let block = Block::default()
            .title(format!(
                " Inventory (a-{}): Page {}/{} ",
                (b'a' + (self.max_items_per_page - 1) as u8) as char,
                self.page + 1,
                total_pages.max(1)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        // 3. 计算当前页物品范围
        let start_idx = self.page * self.max_items_per_page;
        let end_idx = (start_idx + self.max_items_per_page).min(items.len());
        let current_page_items = &items[start_idx..end_idx];

        // 4. 构建物品列表（带选择高亮）
        let list_items: Vec<ListItem> = current_page_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = start_idx + i == self.selected_index;
                let bg_color = if is_selected {
                    Color::DarkGray
                } else {
                    Color::Reset
                };

                // 根据物品类型设置颜色（参考像素地牢配色）
                let (prefix, text_color) = match item.item_type {
                    ItemType::Weapon => ("W:", Color::LightRed),
                    ItemType::Armor => ("A:", Color::LightBlue),
                    ItemType::Potion => ("P:", Color::LightGreen),
                    ItemType::Scroll => ("S:", Color::LightYellow),
                    ItemType::Food => ("F:", Color::LightMagenta),
                    _ => ("", Color::White),
                };

                let line = Spans::from(vec![
                    Span::styled(
                        format!("{} ", (b'a' + i as u8) as char),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        prefix,
                        Style::default().fg(text_color),
                    ),
                    Span::styled(
                        item.name.clone(),
                        Style::default()
                            .fg(text_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" x{}", item.quantity),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line).style(Style::default().bg(bg_color))
            })
            .collect();

        // 5. 渲染物品列表
        let list = List::new(list_items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.selected_index);

        // 6. 渲染物品详细信息（在底部区域）
        if let Some(selected_item) = items.get(self.selected_index) {
            let desc_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(4),
                width: area.width,
                height: 3,
            };

            self.render_item_details(f, desc_area, selected_item);
        }
    }

    /// 渲染物品详细信息（参考像素地牢底部说明栏）
    fn render_item_details<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        item: &Item,
    ) {
        let desc_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let text = vec![
            Spans::from(Span::styled(
                format!("{}", item.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Spans::from(Span::styled(
                item.description.clone(),
                Style::default().fg(Color::Gray),
            )),
        ];

        let paragraph = Paragraph::new(text)
            .block(desc_block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, area);
    }

    /// 处理键盘输入（物品选择/翻页）
    pub fn handle_input(&mut self, key: KeyCode, item_count: usize) -> Option<usize> {
        match key {
            // 物品选择（a-j对应0-9）
            KeyCode::Char('a') => Some(0),
            KeyCode::Char('b') => Some(1),
            KeyCode::Char('c') => Some(2),
            KeyCode::Char('d') => Some(3),
            KeyCode::Char('e') => Some(4),
            KeyCode::Char('f') => Some(5),
            KeyCode::Char('g') => Some(6),
            KeyCode::Char('h') => Some(7),
            KeyCode::Char('i') => Some(8),
            KeyCode::Char('j') => Some(9),
            
            // 翻页控制
            KeyCode::Right => {
                self.page = (self.page + 1).min((item_count / self.max_items_per_page).saturating_sub(1));
                None
            }
            KeyCode::Left => {
                self.page = self.page.saturating_sub(1);
                None
            }
            
            // 方向键选择
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1).min(item_count.saturating_sub(1));
                self.update_page();
                None
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                self.update_page();
                None
            }
            
            _ => None,
        }
    }

    /// 自动调整页码保证选中项可见
    fn update_page(&mut self) {
        self.page = self.selected_index / self.max_items_per_page;
    }
}

/// 物品类型颜色映射（扩展像素地牢经典配色）
pub fn item_color(item_type: ItemType) -> Color {
    match item_type {
        ItemType::Weapon => Color::LightRed,
        ItemType::Armor => Color::LightBlue,
        ItemType::Potion => Color::LightGreen,
        ItemType::Scroll => Color::LightYellow,
        ItemType::Food => Color::LightMagenta,
        ItemType::Ring => Color::LightCyan,
        ItemType::Wand => Color::LightYellow,
        ItemType::Misc => Color::Gray,
    }
}
