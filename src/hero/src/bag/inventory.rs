//src/hero/bag/inventory.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// 库存系统错误类型
#[derive(Debug, Error, PartialEq)]
pub enum InventoryError {
    #[error("背包已满")]
    Full,
    #[error("物品堆叠已达上限")]
    StackLimit,
    #[error("物品不可堆叠")]
    NotStackable,
    #[error("未知物品类型")]
    UnknownItemType,
}

/// 物品特性约束（根据游戏机制扩展）
pub trait ItemTrait: Eq + std::hash::Hash + Clone + Serialize {
    /// 是否可堆叠（药水/卷轴可堆叠，武器不可）
    fn is_stackable(&self) -> bool;

    /// 最大堆叠数量（默认10，参考游戏设定）
    fn max_stack(&self) -> u32 {
        20
    }

    /// 终端显示名称（用于TUI渲染）
    fn display_name(&self) -> &'static str;

    /// 物品分类（用于自动整理）
    fn category(&self) -> ItemCategory;
}

/// 物品分类（影响背包整理逻辑）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Potion,
    Scroll,
    Food,
    Ring,
    // ...其他分类
}

/// 通用库存系统（严格遵循游戏机制）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory<T: ItemTrait> {
    slots: HashMap<T, u32>, // 物品->数量映射
    capacity: usize,        // 最大槽位数
    total_items: usize,     // 当前物品总数（含堆叠）
}

impl<T: ItemTrait> Inventory<T> {
    /// 创建新库存（参考游戏初始背包容量）
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: HashMap::with_capacity(capacity),
            capacity,
            total_items: 0,
        }
    }

    /// 添加物品（智能堆叠处理）
    pub fn add(&mut self, item: T) -> Result<(), InventoryError> {
        // 检查基础容量
        if self.slots.len() >= self.capacity && !self.slots.contains_key(&item) {
            return Err(InventoryError::Full);
        }

        // 处理堆叠逻辑
        if let Some(count) = self.slots.get_mut(&item) {
            if !item.is_stackable() {
                return Err(InventoryError::NotStackable);
            }
            if *count >= item.max_stack() {
                return Err(InventoryError::StackLimit);
            }
            *count += 1;
        } else {
            self.slots.insert(item.clone(), 1);
        }

        self.total_items += 1;
        Ok(())
    }

    /// 移除物品（保持游戏内物品消失效果）
    pub fn remove(&mut self, item: &T) -> Result<(), InventoryError> {
        match self.slots.get_mut(item) {
            Some(count) => {
                *count -= 1;
                if *count == 0 {
                    self.slots.remove(item);
                }
                self.total_items -= 1;
                Ok(())
            }
            None => Err(InventoryError::UnknownItemType),
        }
    }

    /// 获取物品数量（用于TUI状态栏显示）
    pub fn count(&self, item: &T) -> u32 {
        self.slots.get(item).copied().unwrap_or(0)
    }

    /// 背包整理（按游戏内分类排序）
    pub fn organize(&mut self) {
        let mut items: Vec<_> = self.slots.drain().collect();
        items.sort_by(|(a, _), (b, _)| {
            a.category()
                .cmp(&b.category())
                .then_with(|| a.display_name().cmp(b.display_name()))
        });
        self.slots = items.into_iter().collect();
    }

    /// 终端渲染格式（支持颜色代码）
    pub fn render(&self) -> Vec<String> {
        self.slots
            .iter()
            .map(|(item, count)| {
                format!(
                    "[{}] {} x{}",
                    item.category().symbol(), // 分类符号
                    item.display_name(),
                    count
                )
            })
            .collect()
    }
}
