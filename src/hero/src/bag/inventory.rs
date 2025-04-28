// src/hero/src/bag/inventory.rs
use bincode::{Decode, Encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, convert::TryInto, sync::Arc};
use thiserror::Error;

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
    #[error("数量转换失败")] // FIXED: 新增错误类型
    QuantityConversion,
}

/// 库存槽位（支持智能指针）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
enum InventorySlot<T: ItemTrait + Serialize + DeserializeOwned> {
    Single(Arc<T>),
    Stackable(Arc<T>, u32),
}

/// 优化后的库存系统
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Inventory<T: ItemTrait + Serialize + DeserializeOwned> {
    slots: Vec<InventorySlot<T>>,
    stack_map: HashMap<u64, Vec<usize>>, // 堆叠索引加速
    capacity: usize,
}

impl<T: ItemTrait + Serialize + DeserializeOwned> Inventory<T> {
    /// 创建新库存
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            stack_map: HashMap::new(),
            capacity,
        }
    }

    /// 安全添加物品（带类型转换检查）
    pub fn add(&mut self, item: T) -> Result<(), InventoryError> {
        let item = Arc::new(item);
        if item.is_stackable() {
            let max_stack = item.max_stack();
            let stack_id = item.stacking_id();
            if stack_id == 0 {
                return Err(InventoryError::UnknownItemType);
            }

            if let Some(indices) = self.stack_map.get_mut(&stack_id) {
                // 反向遍历优先检查最近添加的堆叠
                for &i in indices.iter().rev() {
                    if let InventorySlot::Stackable(_, ref mut count) = &mut self.slots[i] {
                        if *count < max_stack {
                            *count += 1;

                            // 自动清理已满堆叠的索引
                            if *count == max_stack {
                                indices.retain(|&x| x != i);
                            }
                            return Ok(());
                        }
                    }
                }
            }

            // 容量检查
            if self.slots.len() >= self.capacity {
                return Err(InventoryError::Full);
            }

            // 更新索引
            let new_index = self.slots.len();
            self.slots.push(InventorySlot::Stackable(item.clone(), 1));
            self.stack_map.entry(stack_id).or_default().push(new_index);
        } else {
            // 安全类型转换
            let quantity: usize = 1
                .try_into()
                .map_err(|_| InventoryError::QuantityConversion)?;

            if self.slots.len() + quantity > self.capacity {
                return Err(InventoryError::Full);
            }

            self.slots.push(InventorySlot::Single(item));
        }
        Ok(())
    }

    /// 批量添加优化实现
    pub fn add_multiple(&mut self, item: T, quantity: u32) -> Result<(), InventoryError> {
        let item = Arc::new(item);
        let stack_id = item.stacking_id();

        let quantity_usize = quantity
            .try_into()
            .map_err(|_| InventoryError::QuantityConversion)?;

        if item.is_stackable() {
            let max_stack = item.max_stack();
            let mut remaining = quantity;

            // 增强的堆叠填充逻辑
            if let Some(indices) = self.stack_map.get_mut(&stack_id) {
                let mut i = 0;
                while i < indices.len() {
                    let index = indices[i];
                    if let InventorySlot::Stackable(_, ref mut count) = self.slots[index] {
                        let available = max_stack - *count;
                        if available > 0 {
                            let add = remaining.min(available);
                            *count += add;
                            remaining -= add;

                            if *count == max_stack {
                                indices.remove(i);
                                continue;
                            }
                        }
                    }
                    i += 1;
                }
            }

            // 预计算所需空间
            let needed_slots = (remaining + max_stack - 1) / max_stack;
            if self.slots.len() + needed_slots as usize > self.capacity {
                return Err(InventoryError::Full);
            }

            // 批量添加优化
            let new_items = (remaining + max_stack - 1) / max_stack;
            self.slots.reserve(new_items as usize);

            let start_index = self.slots.len();
            let mut current = remaining;
            while current > 0 {
                let amount = current.min(max_stack);
                self.slots
                    .push(InventorySlot::Stackable(item.clone(), amount));
                current -= amount;
            }
            self.update_stack_map(stack_id, start_index..self.slots.len());
        } else {
            // 内存预分配
            if self.slots.len() + quantity_usize > self.capacity {
                return Err(InventoryError::Full);
            }
            self.slots.reserve(quantity_usize);

            for _ in 0..quantity {
                self.slots.push(InventorySlot::Single(item.clone()));
            }
        }
        Ok(())
    }

    /// 更新后的移除方法
    pub fn remove(&mut self, index: usize) -> Result<Arc<T>, InventoryError> {
        if index >= self.slots.len() {
            return Err(InventoryError::InvalidIndex);
        }

        // 获取堆叠ID并准备更新索引
        let stack_id = match &self.slots[index] {
            InventorySlot::Single(item) => item.stacking_id(),
            InventorySlot::Stackable(item, _) => item.stacking_id(),
        };

        let result = match &mut self.slots[index] {
            InventorySlot::Single(_) => {
                let slot = self.slots.remove(index);
                self.update_indexes_after_removal(index);
                self.cleanup_stack_map(stack_id, index);
                match slot {
                    InventorySlot::Single(item) => Ok(item),
                    _ => unreachable!(),
                }
            }
            InventorySlot::Stackable(item, count) => {
                // 预检查堆叠数量
                if *count == 0 {
                    return Err(InventoryError::InvalidIndex);
                }

                let cloned_item = item.clone();
                *count -= 1; // 减少堆叠数量

                match *count {
                    0 => {
                        // 完全移除空堆叠
                        self.slots.remove(index);
                        self.update_indexes_after_removal(index);
                        self.cleanup_stack_map(stack_id, index);
                    }
                    _ => {
                        // 更新可堆叠索引列表
                        let indices = self.stack_map.entry(stack_id).or_default();

                        // 使用二分查找保持有序插入
                        match indices.binary_search(&index) {
                            Ok(_) => {} // 已存在则保留
                            Err(pos) => indices.insert(pos, index),
                        }
                    }
                }

                Ok(cloned_item)
            }
        };

        result
    }

    /// 移除整个槽位
    pub fn remove_slot(&mut self, index: usize) -> Result<Arc<T>, InventoryError> {
        // FIXED: 返回Arc<T>
        if index >= self.slots.len() {
            return Err(InventoryError::InvalidIndex);
        }

        // 获取堆叠ID并准备更新索引
        let stack_id = match &self.slots[index] {
            InventorySlot::Single(item) => item.stacking_id(),
            InventorySlot::Stackable(item, _) => item.stacking_id(),
        };

        let slot = self.slots.remove(index);
        self.update_indexes_after_removal(index);
        self.cleanup_stack_map(stack_id, index); // FIXED: 增加索引清理

        match slot {
            InventorySlot::Single(item) => Ok(item),
            InventorySlot::Stackable(item, _) => Ok(item),
        }
    }

    /// 整理背包
    pub fn organize(&mut self) {
        // 使用统一的排序逻辑
        self.sort_by(|a, b| match (a, b) {
            (InventorySlot::Single(a), InventorySlot::Single(b))
            | (InventorySlot::Stackable(a, _), InventorySlot::Stackable(b, _)) => a
                .category()
                .cmp(&b.category())
                .then_with(|| b.sort_value().cmp(&a.sort_value())),
            (InventorySlot::Single(_), InventorySlot::Stackable(_, _)) => Ordering::Less,
            (InventorySlot::Stackable(_, _), InventorySlot::Single(_)) => Ordering::Greater,
        });
    }

    /// 自定义排序
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        // 原地排序代替克隆
        self.slots.sort_by(|a, b| match (a, b) {
            (InventorySlot::Single(a), InventorySlot::Single(b))
            | (InventorySlot::Stackable(a, _), InventorySlot::Stackable(b, _)) => compare(a, b),
            (InventorySlot::Single(_), InventorySlot::Stackable(_, _)) => Ordering::Less,
            (InventorySlot::Stackable(_, _), InventorySlot::Single(_)) => Ordering::Greater,
        });

        // 重建索引映射
        let mut new_stack_map = HashMap::new();
        for (new_idx, slot) in self.slots.iter().enumerate() {
            let stack_id = match slot {
                InventorySlot::Single(i) => i.stacking_id(),
                InventorySlot::Stackable(i, _) => i.stacking_id(),
            };
            new_stack_map
                .entry(stack_id)
                .or_insert_with(Vec::new)
                .push(new_idx);
        }
        self.stack_map = new_stack_map;
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

    /// 添加并排序物品
    pub fn add_sorted<F>(&mut self, item: T, compare: F) -> Result<(), InventoryError>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.add(item)?;
        self.sort_by(compare);
        Ok(())
    }

    /// 更新移除位置之后的所有索引
    fn update_indexes_after_removal(&mut self, removed_index: usize) {
    for indices in self.stack_map.values_mut() {
        *indices = indices.iter()
            .filter_map(|&i| {
                if i == removed_index {
                    None
                } else if i > removed_index {
                    Some(i - 1)
                } else {
                    Some(i)
                }
            })
            .collect();
        indices.sort_unstable();
    }
}

    /// 清理特定槽位的索引
    fn cleanup_stack_map(&mut self, stack_id: u64, removed_index: usize) {
        if let Some(indices) = self.stack_map.get_mut(&stack_id) {
            // 二分查找移除索引
            if let Ok(pos) = indices.binary_search(&removed_index) {
                indices.remove(pos);
            }
            // 清理空条目
            if indices.is_empty() {
                self.stack_map.remove(&stack_id);
            }
        }
    }

    /// 更新stack_map索引（用于批量添加）
    fn update_stack_map(&mut self, stack_id: u64, new_indices: impl Iterator<Item = usize>) {
        let entry = self.stack_map.entry(stack_id).or_default();
        entry.extend(new_indices);
        entry.sort(); // 保持有序便于二分查找
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
}
