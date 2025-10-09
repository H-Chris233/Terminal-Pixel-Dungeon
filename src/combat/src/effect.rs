//src/combat/src/effect.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{Display, EnumIter, EnumString};
use ratatui::style::{Color, Style};

/// 效果实例（现在包含视觉状态）
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Effect {
    effect_type: EffectType,
    turns: u32,      // 剩余回合数
    intensity: u8,   // 效果强度
}

impl Effect {
    /// 创建新效果（默认强度3）
    pub fn new(effect_type: EffectType, turns: u32) -> Self {
        Self {
            effect_type,
            turns,
            intensity: 3,
        }
    }

    pub fn with_intensity(effect_type: EffectType, turns: u32, intensity: u8) -> Self {
        Self {
            effect_type,
            turns,
            intensity: intensity.clamp(1, 10),
        }
    }

    /// 检查效果是否已结束
    pub fn is_expired(&self) -> bool {
        self.turns == 0
    }

    /// 更新效果回合数（返回是否仍有效）
    pub fn update(&mut self) -> bool {
        if self.turns > 0 {
            self.turns -= 1;
            !self.is_expired()
        } else {
            false
        }
    }

    /// 获取效果类型
    pub fn effect_type(&self) -> EffectType {
        self.effect_type
    }

    /// 获取剩余回合数
    pub fn turns(&self) -> u32 {
        self.turns
    }

    /// 设置剩余回合数
    pub fn set_turns(&mut self, turns: u32) {
        self.turns = turns;
    }

    /// 获取效果强度
    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    /// 统一伤害计算逻辑（每回合必定触发）
    pub fn damage(&self) -> u32 {
        match self.effect_type {
            EffectType::Burning => (self.intensity as u32) * 2,
            EffectType::Poison => (self.intensity as u32) * 3,
            EffectType::Bleeding => (self.intensity as u32) * 4,
            _ => 0,
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
        !self.is_stackable() && !matches!(
            self.effect_type,
            EffectType::MindVision | EffectType::Invisibility
        )
    }

    /// 获取效果描述（用于UI显示）
    pub fn description(&self) -> String {
        let base = match self.effect_type {
            EffectType::Burning => format!("燃烧(每回合-{}HP)", self.damage()),
            EffectType::Poison => format!("中毒(每回合-{}HP)", self.damage()),
            EffectType::Bleeding => format!("流血(每回合-{}HP)", self.damage()), // 修改为每回合触发
            EffectType::Paralysis => "麻痹无法行动".to_string(),
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

        if self.turns > 0 {
            format!("{} (剩余{}回合)", base, self.turns)
        } else {
            base
        }
    }

    
    /// 获取当前视觉样式（始终可见）
    pub fn current_style(&self) -> Style {
        self.effect_type.visual_style().to_style()
    }

    /// 获取始终可见的覆盖字符
    pub fn overlay_char(&self) -> Option<char> {
        self.visual_config().overlay_char()
    }

    /// 获取状态栏显示样式
    pub fn status_style(&self) -> Style {
        Style::default()
            .fg(self.effect_type.status_color().into())
            .add_modifier(tui::style::Modifier::BOLD)
    }
    
    /// 获取视觉配置的方法
    pub fn visual_config(&self) -> VisualEffect {
        self.effect_type.visual_style()
    }
    
    /// 更新后的视觉效果状态方法（供UI查询刷新需求）
    pub fn should_blink(&self, current_turn: u64) -> bool {
    let config = self.visual_config();
    if config.blink_interval() == 0 {
        return false;
    }
    current_turn % config.blink_interval() == 0
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
    Hash,
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
    pub blink_interval: u64, // 闪烁间隔（回合数）
}

impl VisualEffect {
    pub fn to_style(&self) -> Style {
        Style::default()
            .fg(self.fg_color.clone().into())
            .bg(self.bg_color.clone().into())
    }
    
    /// 获取闪烁间隔（供UI模块调用）
    pub fn blink_interval(&self) -> u64 {
        self.blink_interval
    }

    /// 获取前景色
    pub fn foreground(&self) -> &SerializableColor {
        &self.fg_color
    }

    /// 获取背景色 
    pub fn background(&self) -> &SerializableColor {
        &self.bg_color
    }
    
    pub fn overlay_char(&self) -> Option<char> {
        self.overlay_char
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
                blink_interval: 1, // 每回合闪烁
            },
            EffectType::Poison => VisualEffect {
                fg_color: SerializableColor::Green,
                bg_color: SerializableColor::Black,
                overlay_char: Some('☠'),
                blink_interval: 2, // 每2回合闪烁
            },
            EffectType::Paralysis => VisualEffect {
                fg_color: SerializableColor::Yellow,
                bg_color: SerializableColor::Black,
                overlay_char: Some('⚡'),
                blink_interval: 1,
            },
            EffectType::Bleeding => VisualEffect {
                fg_color: SerializableColor::Rgb(139, 0, 0),
                bg_color: SerializableColor::Black,
                overlay_char: Some('🩸'),
                blink_interval: 0, // 不闪烁
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
                blink_interval: 1,
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
                blink_interval: 1,
            },
            EffectType::MindVision => VisualEffect {
                fg_color: SerializableColor::Magenta,
                bg_color: SerializableColor::Black,
                overlay_char: Some('👁'),
                blink_interval: 2,
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
                blink_interval: 1,
            },
            EffectType::Fury => VisualEffect {
                fg_color: SerializableColor::Red,
                bg_color: SerializableColor::Black,
                overlay_char: Some('💢'),
                blink_interval: 1,
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
                blink_interval: 1,
            },
            EffectType::Darkness => VisualEffect {
                fg_color: SerializableColor::DarkGray,
                bg_color: SerializableColor::Black,
                overlay_char: Some('🌑'),
                blink_interval: 0,
            },
            EffectType::Rooted => VisualEffect {
                fg_color: SerializableColor::Rgb(101, 67, 33),
                bg_color: SerializableColor::Black,
                overlay_char: Some('🌿'),
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
            EffectType::Rooted => SerializableColor::Rgb(101, 67, 33),
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
            Color::Indexed(n) => Self::Rgb(
                ((n >> 16) & 0xFF) as u8,
                ((n >> 8) & 0xFF) as u8,
                (n & 0xFF) as u8
            ),
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
