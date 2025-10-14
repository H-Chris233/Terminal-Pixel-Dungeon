/// Hero 模块的事件定义
/// 这些事件会被收集并由主项目发布到事件总线
#[derive(Debug, Clone)]
pub enum HeroEvent {
    /// 物品拾取事件
    ItemPickedUp {
        entity: u32,
        item_name: String,
    },

    /// 物品丢弃事件
    ItemDropped {
        entity: u32,
        item_name: String,
    },

    /// 物品使用事件
    ItemUsed {
        entity: u32,
        item_name: String,
        effect: String,
    },

    /// 物品装备事件
    ItemEquipped {
        entity: u32,
        item_name: String,
        slot: String,
    },

    /// 物品卸下事件
    ItemUnequipped {
        entity: u32,
        item_name: String,
        slot: String,
    },

    /// 英雄移动事件
    Moved {
        entity: u32,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },

    /// 等级提升事件
    LevelUp {
        entity: u32,
        new_level: u32,
    },

    /// 陷阱触发事件
    TrapTriggered {
        entity: u32,
        trap_type: String,
    },

    /// 状态效果应用事件
    StatusApplied {
        entity: u32,
        status: String,
        duration: u32,
    },

    /// 状态效果移除事件
    StatusRemoved {
        entity: u32,
        status: String,
    },
}

/// 操作结果，包含事件信息
#[derive(Debug, Default)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
    pub events: Vec<HeroEvent>,
}

impl ActionResult {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            message: None,
            events: Vec::new(),
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn add_event(&mut self, event: HeroEvent) {
        self.events.push(event);
    }

    pub fn success() -> Self {
        Self::new(true)
    }

    pub fn failure() -> Self {
        Self::new(false)
    }
}
