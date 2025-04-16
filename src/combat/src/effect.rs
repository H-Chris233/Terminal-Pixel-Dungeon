// src/combat/src/effect.rs
use bincode::{Decode, Encode};
use items::Item;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use strum::{Display, EnumIter, EnumString};
use tui::style::{Color, Style};

/// 效果实例（现在包含视觉状态）
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Effect {
    effect_type: EffectType,
    duration: Duration,
    intensity: u8,
    source: EffectSource,
    last_blink: u64, // 最后闪烁时间(ms)
    visible: bool,   // 当前是否可见（用于闪烁效果）
}

/// 效果来源（用于伤害计算和免疫判断）
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum EffectSource {
    Player,
    Enemy,
    Environment,
    Item(Item), // 物品名称
}

impl Effect {
    /// 创建新效果（默认强度3）
    pub fn new(effect_type: EffectType, duration: Duration, source: EffectSource) -> Self {
        Self {
            effect_type,
            duration,
            intensity: 3,
            source,
            last_blink: 0, // 新增字段
            visible: true, // 新增字段
        }
    }

    pub fn with_intensity(
        effect_type: EffectType,
        duration: Duration,
        intensity: u8,
        source: EffectSource,
    ) -> Self {
        Self {
            effect_type,
            duration,
            intensity: intensity.clamp(1, 10),
            source,
            last_blink: 0, // 新增字段
            visible: true, // 新增字段
        }
    }

    /// 检查效果是否已结束
    pub fn is_expired(&self) -> bool {
        self.duration.as_secs() == 0
    }

    /// 更新效果持续时间（返回是否仍有效）
    pub fn update(&mut self, elapsed: Duration) -> bool {
        if let Some(new_duration) = self.duration.checked_sub(elapsed) {
            self.duration = new_duration;
            !self.is_expired()
        } else {
            self.duration = Duration::ZERO;
            false
        }
    }

    /// 获取效果类型
    pub fn effect_type(&self) -> EffectType {
        self.effect_type
    }

    /// 获取剩余持续时间
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// 获取效果强度
    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    /// 获取效果来源
    pub fn source(&self) -> &EffectSource {
        &self.source
    }

    /// 计算效果造成的伤害（基于类型和强度）
    pub fn calculate_damage(&self) -> u32 {
        match self.effect_type {
            EffectType::Burning => (self.intensity as u32) * 2,
            EffectType::Poison => (self.intensity as u32) * 3,
            EffectType::Bleeding => (self.intensity as u32) * 4,
            _ => 0, // 其他效果不直接造成伤害
        }
    }

    /// 效果是否可以叠加（相同类型）
    pub fn is_stackable(&self) -> bool {
        matches!(
            self.effect_type,
            EffectType::Burning | EffectType::Poison | EffectType::Bleeding
        )
    }

    /// 效果是否会被相同类型覆盖（非叠加效果）
    pub fn is_overwritable(&self) -> bool {
        !self.is_stackable()
            && !matches!(
                self.effect_type,
                EffectType::MindVision | EffectType::Invisibility
            )
    }

    /// 获取效果描述（用于UI显示）
    pub fn description(&self) -> String {
        let base = match self.effect_type {
            EffectType::Burning => format!("燃烧中(每回合-{}HP)", self.calculate_damage()),
            EffectType::Poison => format!("中毒(每回合-{}HP)", self.calculate_damage()),
            EffectType::Paralysis => "麻痹无法行动".to_string(),
            EffectType::Bleeding => format!("流血(移动时-{}HP)", self.calculate_damage()),
            EffectType::Invisibility => "隐身中".to_string(),
            EffectType::Levitation => "漂浮中".to_string(),
            EffectType::Slow => "减速".to_string(),
            EffectType::Haste => "加速".to_string(),
            EffectType::MindVision => "灵视效果".to_string(),
            EffectType::AntiMagic => "魔法抗性提升".to_string(),
            EffectType::Barkskin => "防御提升".to_string(),
            EffectType::Combo => "连击准备".to_string(),
            EffectType::Fury => "狂暴状态".to_string(),
            EffectType::Ooze => "被淤泥覆盖".to_string(),
            EffectType::Frost => "身体冻僵".to_string(),
            EffectType::Light => "发光中".to_string(),
            EffectType::Darkness => "视线受阻".to_string(),
            EffectType::Rooted => "根系缠绕(无法移动)".to_string(),
        };

        if self.duration.as_secs() > 0 {
            format!("{} (剩余{}回合)", base, self.duration.as_secs())
        } else {
            base
        }
    }
    /// 更新视觉效果状态（返回是否需要重绘）
    pub fn update_visual(&mut self, elapsed_ms: u64) -> bool {
        self.last_blink += elapsed_ms;
        let visual = self.effect_type.visual_style();

        if visual.blink_interval > 0 {
            if self.last_blink >= visual.blink_interval {
                self.last_blink = 0;
                self.visible = !self.visible;
                return true;
            }
        }
        false
    }

    /// 获取当前视觉样式
    pub fn current_style(&self) -> Style {
        let visual = self.effect_type.visual_style();
        let mut style = visual.to_style();
        if !self.visible {
            style = style.fg(Color::Reset).bg(Color::Reset);
        }
        style
    }

    /// 获取覆盖字符（如果有）
    pub fn overlay_char(&self) -> Option<char> {
        if self.visible {
            self.effect_type.visual_style().overlay_char
        } else {
            None
        }
    }

    /// 获取状态栏显示样式
    pub fn status_style(&self) -> Style {
        Style::default()
            .fg(self.effect_type.status_color().into())
            .add_modifier(tui::style::Modifier::BOLD)
    }
}

/// 效果类型（包含视觉标记信息）
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Display,
    EnumIter,
    EnumString,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[strum(serialize_all = "snake_case")]
pub enum EffectType {
    #[strum(serialize = "burning(燃烧)")]
    Burning, // 红色火焰效果
    #[strum(serialize = "poison(中毒)")]
    Poison, // 绿色毒雾效果
    #[strum(serialize = "paralysis(麻痹)")]
    Paralysis, // 黄色电击效果
    #[strum(serialize = "bleeding(流血)")]
    Bleeding, // 深红色血滴效果
    #[strum(serialize = "invisibility(隐身)")]
    Invisibility, // 半透明效果
    #[strum(serialize = "levitation(漂浮)")]
    Levitation, // 淡蓝色上升波纹
    #[strum(serialize = "slow(减速)")]
    Slow, // 灰色粘液效果
    #[strum(serialize = "haste(加速)")]
    Haste, // 亮绿色流光效果
    #[strum(serialize = "mind_vision(灵视)")]
    MindVision, // 紫色光环
    #[strum(serialize = "anti_magic(反魔法)")]
    AntiMagic, // 深蓝色符文
    #[strum(serialize = "barkskin(树皮)")]
    Barkskin, // 棕色树皮纹理
    #[strum(serialize = "combo(连击)")]
    Combo, // 橙色连击计数
    #[strum(serialize = "fury(狂暴)")]
    Fury, // 红色狂暴气息
    #[strum(serialize = "ooze(淤泥)")]
    Ooze, // 深绿色粘液
    #[strum(serialize = "frost(冰冻)")]
    Frost, // 浅蓝色冰晶
    #[strum(serialize = "light(光明)")]
    Light, // 亮黄色发光
    #[strum(serialize = "darkness(黑暗)")]
    Darkness, // 深紫色迷雾
    #[strum(serialize = "rooted(根系缠绕)")]
    Rooted,
}

/// 视觉效果配置
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct VisualEffect {
    pub fg_color: SerializableColor,
    pub bg_color: SerializableColor,
    pub overlay_char: Option<char>,
    pub blink_interval: u64,
}

impl VisualEffect {
    pub fn to_style(&self) -> Style {
        Style::default()
            .fg(self.fg_color.clone().into())
            .bg(self.bg_color.clone().into())
    }
}

impl EffectType {
    /// 获取效果的视觉配置
    pub fn visual_style(&self) -> VisualEffect {
        match self {
            EffectType::Burning => VisualEffect {
                fg_color: SerializableColor::Red,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🔥'),
                blink_interval: 300,
            },
            EffectType::Poison => VisualEffect {
                fg_color: SerializableColor::Green,
                bg_color: SerializableColor::Black,
                overlay_char: Some('☠'),
                blink_interval: 500,
            },
            EffectType::Paralysis => VisualEffect {
                fg_color: SerializableColor::Yellow,
                bg_color: SerializableColor::Black,
                overlay_char: Some('⚡'),
                blink_interval: 200,
            },
            EffectType::Bleeding => VisualEffect {
                fg_color: SerializableColor::Rgb(139, 0, 0),
                bg_color: SerializableColor::Black,
                overlay_char: Some('🩸'),
                blink_interval: 0,
            },
            EffectType::Invisibility => VisualEffect {
                fg_color: SerializableColor::Gray,
                bg_color: SerializableColor::Black,
                overlay_char: Some('👻'),
                blink_interval: 0,
            },
            EffectType::Levitation => VisualEffect {
                fg_color: SerializableColor::LightBlue,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🔼'),
                blink_interval: 400,
            },
            EffectType::Slow => VisualEffect {
                fg_color: SerializableColor::Gray,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🕸'),
                blink_interval: 0,
            },
            EffectType::Haste => VisualEffect {
                fg_color: SerializableColor::LightGreen,
                bg_color: SerializableColor::Black,
                overlay_char: Some('⚡'),
                blink_interval: 100,
            },
            EffectType::MindVision => VisualEffect {
                fg_color: SerializableColor::Magenta,
                bg_color: SerializableColor::Black,
                overlay_char: Some('👁'),
                blink_interval: 600,
            },
            EffectType::AntiMagic => VisualEffect {
                fg_color: SerializableColor::Blue,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🛡'),
                blink_interval: 0,
            },
            EffectType::Barkskin => VisualEffect {
                fg_color: SerializableColor::Rgb(139, 69, 19),
                bg_color: SerializableColor::Black,
                overlay_char: Some('🌲'),
                blink_interval: 0,
            },
            EffectType::Combo => VisualEffect {
                fg_color: SerializableColor::LightYellow,
                bg_color: SerializableColor::Black,
                overlay_char: Some('➰'),
                blink_interval: 150,
            },
            EffectType::Fury => VisualEffect {
                fg_color: SerializableColor::Red,
                bg_color: SerializableColor::Black,
                overlay_char: Some('💢'),
                blink_interval: 200,
            },
            EffectType::Ooze => VisualEffect {
                fg_color: SerializableColor::Rgb(0, 100, 0),
                bg_color: SerializableColor::Black,
                overlay_char: Some('🟢'),
                blink_interval: 0,
            },
            EffectType::Frost => VisualEffect {
                fg_color: SerializableColor::LightCyan,
                bg_color: SerializableColor::Black,
                overlay_char: Some('❄'),
                blink_interval: 0,
            },
            EffectType::Light => VisualEffect {
                fg_color: SerializableColor::Yellow,
                bg_color: SerializableColor::Black,
                overlay_char: Some('✨'),
                blink_interval: 300,
            },
            EffectType::Darkness => VisualEffect {
                fg_color: SerializableColor::DarkGray,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🌑'),
                blink_interval: 0,
            },
            EffectType::Rooted => VisualEffect {
                fg_color: SerializableColor::Rgb(101, 67, 33), // 棕色
                bg_color: SerializableColor::Black,
                overlay_char: Some('🌿'), // 使用植物符号
                blink_interval: 0,
            },
        }
    }

    /// 获取效果的基础颜色（用于状态栏显示）
    pub fn status_color(&self) -> SerializableColor {
        match self {
            EffectType::Burning => SerializableColor::Red,
            EffectType::Poison => SerializableColor::Green,
            EffectType::Paralysis => SerializableColor::Yellow,
            EffectType::Bleeding => SerializableColor::Rgb(139, 0, 0),
            EffectType::Invisibility => SerializableColor::Gray,
            EffectType::Levitation => SerializableColor::LightBlue,
            EffectType::Slow => SerializableColor::Gray,
            EffectType::Haste => SerializableColor::LightGreen,
            EffectType::MindVision => SerializableColor::Magenta,
            EffectType::AntiMagic => SerializableColor::Blue,
            EffectType::Barkskin => SerializableColor::Rgb(139, 69, 19),
            EffectType::Combo => SerializableColor::LightYellow,
            EffectType::Fury => SerializableColor::Red,
            EffectType::Ooze => SerializableColor::Rgb(0, 100, 0),
            EffectType::Frost => SerializableColor::LightCyan,
            EffectType::Light => SerializableColor::Yellow,
            EffectType::Darkness => SerializableColor::DarkGray,
            EffectType::Rooted => SerializableColor::Rgb(101, 67, 33), // 棕色
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum SerializableColor {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    Rgb(u8, u8, u8),
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::Red,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::White => Self::White,
            Color::Gray => Self::Gray,
            Color::DarkGray => Self::DarkGray,
            Color::LightRed => Self::LightRed,
            Color::LightGreen => Self::LightGreen,
            Color::LightYellow => Self::LightYellow,
            Color::LightBlue => Self::LightBlue,
            Color::LightMagenta => Self::LightMagenta,
            Color::LightCyan => Self::LightCyan,
            Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
            Color::Indexed(_) => todo!(),
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        match color {
            SerializableColor::Reset => Self::Reset,
            SerializableColor::Black => Self::Black,
            SerializableColor::Red => Self::Red,
            SerializableColor::Green => Self::Green,
            SerializableColor::Yellow => Self::Yellow,
            SerializableColor::Blue => Self::Blue,
            SerializableColor::Magenta => Self::Magenta,
            SerializableColor::Cyan => Self::Cyan,
            SerializableColor::White => Self::White,
            SerializableColor::Gray => Self::Gray,
            SerializableColor::DarkGray => Self::DarkGray,
            SerializableColor::LightRed => Self::LightRed,
            SerializableColor::LightGreen => Self::LightGreen,
            SerializableColor::LightYellow => Self::LightYellow,
            SerializableColor::LightBlue => Self::LightBlue,
            SerializableColor::LightMagenta => Self::LightMagenta,
            SerializableColor::LightCyan => Self::LightCyan,
            SerializableColor::Rgb(r, g, b) => Self::Rgb(r, g, b),
        }
    }
}
