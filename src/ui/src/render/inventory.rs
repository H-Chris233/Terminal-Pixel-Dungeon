//src/ui/render/inventory.rs
use items::{Item, ItemKind};
use hero::Hero;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 物品栏渲染器（支持分页和动态高亮）
pub struct InventoryRenderer {
    pub page: usize,               // 当前页码
    pub selected_index: usize,     // 选中项索引
    pub scroll_offset: usize,      // 滚动偏移
    pub max_items_per_page: usize, // 每页最大显示数
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
    pub fn render(&mut self, f: &mut Frame, area: Rect, hero: &Hero) {
        // 1. 直接访问Vec<Item> (Inventory是Vec<Item>的别名)
        let items: Vec<items::Item> = Vec::new();
        // 2. 分页计算（增加防零除保护）
        let total_pages = if self.max_items_per_page == 0 {
            1
        } else {
            (items.len().saturating_sub(1) / self.max_items_per_page) + 1
        };
        self.page = self.page.min(total_pages.saturating_sub(1));

        // 3. 创建区块（保持原样）
        let block = Block::default()
            .title(format!(
                " Inventory (a-{}): Page {}/{} ",
                (b'a' + (self.max_items_per_page - 1) as u8) as char,
                self.page + 1,
                total_pages.max(1)
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        // 4. 当前页物品范围（优化边界检查）
        let start_idx = self.page * self.max_items_per_page;
        let end_idx = (start_idx + self.max_items_per_page).min(items.len());

        // 5. 构建列表项（修复颜色引用）
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));

        let list_items = items[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = (start_idx + i) == self.selected_index;

                // 使用RGB值替代不存在的DarkGray
                let bg_color = if is_selected {
                    Color::Rgb(80, 80, 80)
                } else {
                    Color::Reset
                };

                // 物品类型颜色映射
                let (prefix, color) = match &item.kind {
                    items::ItemKind::Weapon(_) => ("W:", Color::LightRed),
                    items::ItemKind::Armor(_) => ("A:", Color::LightBlue),
                    items::ItemKind::Potion(_) => ("P:", Color::LightGreen),
                    items::ItemKind::Scroll(_) => ("S:", Color::LightYellow),
                    items::ItemKind::Food(_) => ("F:", Color::LightMagenta),
                    items::ItemKind::Ring(_) => ("R:", Color::LightCyan),
                    items::ItemKind::Wand(_) => ("Wn:", Color::LightYellow),
                    items::ItemKind::Seed(_) => ("Se:", Color::Green),
                    items::ItemKind::Stone(_) => ("St:", Color::Gray),
                    items::ItemKind::Misc(_) => ("M:", Color::Gray),
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", (b'a' + i as u8) as char),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(
                        item.name(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" x{}", item.quantity),
                        Style::default().fg(Color::Rgb(120, 120, 120)), // 替代DarkGray
                    ),
                ]))
                .style(Style::default().bg(bg_color))
            });

        // 6. 渲染列表（使用正确的stateful_widget）
        let list = List::new(list_items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(80, 80, 80))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut list_state);

        // 7. 渲染选中物品详情（增加空检查）
        if let Some(item) = items.get(self.selected_index) {
            let desc_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(4),
                width: area.width,
                height: 3,
            };
            self.render_item_details(f, desc_area, item);
        }
    }

    /// 渲染物品详细信息（参考像素地牢底部说明栏）
    fn render_item_details(&self, f: &mut Frame, area: Rect, item: &Item) {
        let desc_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let text = vec![
            Line::from(Span::styled(
                item.name(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
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
                self.page =
                    (self.page + 1).min((item_count / self.max_items_per_page).saturating_sub(1));
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
pub fn item_color(kind: &items::ItemKind) -> Color {
    match kind {
        items::ItemKind::Weapon(_) => Color::LightRed,
        items::ItemKind::Armor(_) => Color::LightBlue,
        items::ItemKind::Potion(_) => Color::LightGreen,
        items::ItemKind::Scroll(_) => Color::LightYellow,
        items::ItemKind::Food(_) => Color::LightMagenta,
        items::ItemKind::Ring(_) => Color::LightCyan,
        items::ItemKind::Wand(_) => Color::LightYellow,
        items::ItemKind::Seed(_) => Color::Green,
        items::ItemKind::Stone(_) => Color::Gray,
        items::ItemKind::Misc(_) => Color::Gray,
    }
}
