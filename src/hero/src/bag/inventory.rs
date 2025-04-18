// src/hero/src/bag/inventory.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

use items::ItemTrait;

/// 库存系统错误类型
#[derive(Debug, Error, PartialEq)]
pub enum InventoryError {
    #[error("背包已满")]
    Full,
    #[error("物品不可堆叠")]
    NotStackable,
    #[error("背包已满")]
    InventoryFull,
    #[error("未知物品类型")]
    UnknownItemType,
    #[error("无效索引")]
    InvalidIndex,
}

/// 通用库存系统（优化实现）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Inventory<T: ItemTrait> {
    slots: Vec<InventorySlot<T>>, // 使用独立槽位设计
    capacity: usize,
}

/// 库存槽位（支持单物品或多数量）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
enum InventorySlot<T: ItemTrait> {
    Single(T),         // 不可堆叠物品
    Stackable(T, u32), // 可堆叠物品（类型+数量）
}

impl<T: ItemTrait> Inventory<T> {
    /// 创建新库存
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// 添加物品（自动处理堆叠逻辑）
    pub fn add(&mut self, item: T) -> Result<(), InventoryError> {
        if self.slots.len() >= self.capacity {
            return Err(InventoryError::Full);
        }

        if item.is_stackable() {
            // 尝试合并到现有堆叠
            for slot in &mut self.slots {
                if let InventorySlot::Stackable(existing, count) = slot {
                    if existing == &item {
                        *count += 1;
                        return Ok(());
                    }
                }
            }
            // 新建堆叠
            self.slots.push(InventorySlot::Stackable(item, 1));
        } else {
            // 添加单物品
            self.slots.push(InventorySlot::Single(item));
        }
        Ok(())
    }

    /// 添加并排序物品
    pub fn add_sorted<F>(&mut self, item: T, compare: F) -> Result<(), InventoryError>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.add(item)?;
        self.sort_by(compare);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items().get(index)
    }
    pub fn remove(&mut self, index: usize) -> Result<T, InventoryError> {
        if index < self.items().len() {
            Ok(self.items().remove(index))
        } else {
            Err(InventoryError::IndexOutOfBounds)
        }
    }

    /// 整理背包（按分类和排序值）
    pub fn organize(&mut self) {
        self.slots.sort_by(|a, b| match (a, b) {
            (InventorySlot::Single(a), InventorySlot::Single(b))
            | (InventorySlot::Stackable(a, _), InventorySlot::Stackable(b, _)) => a
                .category()
                .cmp(&b.category())
                .then_with(|| b.sort_value().cmp(&a.sort_value())),
            _ => Ordering::Equal,
        });
    }

    /// 自定义排序
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.slots.sort_by(|a, b| match (a, b) {
            (InventorySlot::Single(a), InventorySlot::Single(b))
            | (InventorySlot::Stackable(a, _), InventorySlot::Stackable(b, _)) => compare(a, b),
            _ => Ordering::Equal,
        });
    }

    /// 查找物品索引
    pub fn find<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(&T) -> bool,
    {
        self.slots.iter().position(|slot| match slot {
            InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => predicate(item),
        })
    }

    /// 获取所有物品（用于UI渲染）
    pub fn items(&self) -> Vec<(&T, u32)> {
        self.slots
            .iter()
            .map(|slot| match slot {
                InventorySlot::Single(item) => (item, 1),
                InventorySlot::Stackable(item, count) => (item, *count),
            })
            .collect()
    }

    /// 当前物品数量
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> Inventory<T> {}
