// src/hero/src/bag/inventory.rs
use bincode::{Decode, Encode};
use serde::de::DeserializeOwned;
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
    #[error("未知物品类型")]
    UnknownItemType,
    #[error("无效索引")]
    InvalidIndex,
}

/// 通用库存系统（优化实现）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Inventory<T: ItemTrait + PartialEq + Serialize + DeserializeOwned> {
    slots: Vec<InventorySlot<T>>, // 使用独立槽位设计
    capacity: usize,
}

/// 库存槽位（支持单物品或多数量）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
enum InventorySlot<T: ItemTrait + Serialize + DeserializeOwned> {
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
    pub fn add(&mut self, item: T) -> Result<(), InventoryError>
    where
        T: PartialEq, // 添加 PartialEq 约束
    {
        if self.slots.len() >= self.capacity {
            return Err(InventoryError::Full);
        }

        if item.is_stackable() {
            let max_stack = item.max_stack();
            // 查找可堆叠的槽位
            for slot in &mut self.slots {
                if let InventorySlot::Stackable(existing_item, ref mut count) = slot {
                    if existing_item == &item && *count < max_stack {
                        *count += 1;
                        return Ok(());
                    }
                }
            }
            // 没有找到可堆叠的槽位，创建新堆叠
            self.slots.push(InventorySlot::Stackable(item, 1));
        } else {
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
    pub fn remove_stack(&mut self, index: usize) -> Result<T, InventoryError> {
        if index >= self.slots.len() {
            return Err(InventoryError::InvalidIndex);
        }
        let slot = self.slots.remove(index);
        match slot {
            InventorySlot::Single(item) => Ok(item),
            InventorySlot::Stackable(item, _) => Ok(item),
        }
    }

    /// 整理背包（按分类和排序值）
    pub fn organize(&mut self)
    where
        T: Ord, // 添加排序所需的 trait 约束
    {
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


impl<T: ItemTrait> Inventory<T> {
    /// 实现其他不依赖具体类型的通用方法
    /// 例如批量添加的扩展方法：
    
    /// 批量添加物品
    pub fn add_multiple(&mut self, item: T, quantity: u32) -> Result<(), InventoryError> 
    where
        T: Clone,
    {
        if item.is_stackable() {
            let max_stack = item.max_stack();
            let remaining = self.try_add_to_existing(&item, quantity, max_stack)?;
            self.add_new_stacks(item, remaining, max_stack)
        } else {
            if self.slots.len() + quantity as usize > self.capacity {
                return Err(InventoryError::Full);
            }
            for _ in 0..quantity {
                self.slots.push(InventorySlot::Single(item.clone()));
            }
            Ok(())
        }
    }

    /// 尝试添加到现有堆叠
    fn try_add_to_existing(&mut self, item: &T, mut quantity: u32, max_stack: u32) -> Result<u32, InventoryError> {
        for slot in &mut self.slots {
            if let InventorySlot::Stackable(existing, count) = slot {
                if existing == item && *count < max_stack {
                    let available_space = max_stack - *count;
                    let add_amount = quantity.min(available_space);
                    *count += add_amount;
                    quantity -= add_amount;
                    if quantity == 0 {
                        return Ok(0);
                    }
                }
            }
        }
        Ok(quantity)
    }

    /// 添加新堆叠
    fn add_new_stacks(&mut self, item: T, mut remaining: u32, max_stack: u32) -> Result<(), InventoryError> {
        let stacks_needed = (remaining + max_stack - 1) / max_stack;  // 向上取整
        if self.slots.len() + stacks_needed as usize > self.capacity {
            return Err(InventoryError::Full);
        }

        while remaining > 0 {
            let amount = remaining.min(max_stack);
            self.slots.push(InventorySlot::Stackable(item.clone(), amount));
            remaining -= amount;
        }
        Ok(())
    }

    /// 清空库存
    pub fn clear(&mut self) {
        self.slots.clear();
    }
    
    /// 移除单个物品（如果是堆叠则减少数量）
    pub fn remove(&mut self, index: usize) -> Result<T, InventoryError>
    where
        T: Clone,
    {
        // 边界检查
        if index >= self.slots.len() {
            return Err(InventoryError::InvalidIndex);
        }

        // 获取可变引用并匹配槽位类型
        match &mut self.slots[index] {
            // 处理单个物品槽位
            InventorySlot::Single(_) => {
                let slot = self.slots.remove(index);
                if let InventorySlot::Single(item) = slot {
                    Ok(item)
                } else {
                    unreachable!() // 类型已匹配，不可能执行到这里
                }
            }

            // 处理可堆叠物品槽位
            InventorySlot::Stackable(item, ref mut count) => {
                // 数量安全检查
                if *count == 0 {
                    return Err(InventoryError::InvalidIndex);
                }

                // 克隆物品用于返回
                let cloned_item = item.clone();

                // 减少堆叠数量
                *count -= 1;

                // 如果数量归零则移除整个槽位
                if *count == 0 {
                    self.slots.remove(index);
                }

                Ok(cloned_item)
            }
        }
    }
}

