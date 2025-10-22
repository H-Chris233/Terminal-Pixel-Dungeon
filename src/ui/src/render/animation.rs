//! 动画系统
//!
//! 提供UI动画效果，包括：
//! - 淡入淡出效果
//! - 滑动过渡
//! - 脉冲效果
//! - 打字机效果
//! - 战斗动画

use ratatui::style::{Color, Style};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 动画类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationType {
    FadeIn,      // 淡入
    FadeOut,     // 淡出
    SlideUp,     // 向上滑动
    SlideDown,   // 向下滑动
    SlideLeft,   // 向左滑动
    SlideRight,  // 向右滑动
    Pulse,       // 脉冲效果
    Typewriter,  // 打字机效果
    Flash,       // 闪烁效果
    Shake,       // 震动效果
    Bounce,      // 弹跳效果
    Grow,        // 放大效果
    Shrink,      // 缩小效果
}

/// 缓动函数类型
#[derive(Debug, Clone, Copy)]
pub enum EaseType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl EaseType {
    /// 根据进度计算缓动值 (0.0 到 1.0)
    pub fn apply(self, progress: f32) -> f32 {
        match self {
            EaseType::Linear => progress,
            EaseType::EaseIn => progress * progress,
            EaseType::EaseOut => 1.0 - (1.0 - progress).powi(2),
            EaseType::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress).powi(2)
                }
            }
            EaseType::Bounce => {
                if progress < 1.0 / 2.75 {
                    7.5625 * progress * progress
                } else if progress < 2.0 / 2.75 {
                    let p = progress - 1.5 / 2.75;
                    7.5625 * p * p + 0.75
                } else if progress < 2.5 / 2.75 {
                    let p = progress - 2.25 / 2.75;
                    7.5625 * p * p + 0.9375
                } else {
                    let p = progress - 2.625 / 2.75;
                    7.5625 * p * p + 0.984375
                }
            }
            EaseType::Elastic => {
                if progress == 0.0 || progress == 1.0 {
                    progress
                } else {
                    let p = progress * 2.0 - 1.0;
                    -(2.0_f32.powf(10.0 * p) * ((p * 10.0 - 0.75) * (2.0 * std::f32::consts::PI / 3.0)).sin()) / 2.0 + 1.0
                }
            }
        }
    }
}

/// 单个动画实例
#[derive(Debug, Clone)]
pub struct Animation {
    pub animation_type: AnimationType,
    pub duration: Duration,
    pub start_time: Instant,
    pub ease_type: EaseType,
    pub loop_count: Option<u32>, // None = 无限循环
    pub current_loop: u32,
    pub reverse_on_loop: bool,   // 是否在循环时反向播放
    pub is_active: bool,
}

impl Animation {
    /// 创建新动画
    pub fn new(
        animation_type: AnimationType,
        duration: Duration,
        ease_type: EaseType,
    ) -> Self {
        Self {
            animation_type,
            duration,
            start_time: Instant::now(),
            ease_type,
            loop_count: None,
            current_loop: 0,
            reverse_on_loop: false,
            is_active: true,
        }
    }

    /// 创建循环动画
    pub fn looped(
        animation_type: AnimationType,
        duration: Duration,
        ease_type: EaseType,
        loop_count: u32,
    ) -> Self {
        let mut anim = Self::new(animation_type, duration, ease_type);
        anim.loop_count = Some(loop_count);
        anim
    }

    /// 创建无限循环动画
    pub fn infinite(
        animation_type: AnimationType,
        duration: Duration,
        ease_type: EaseType,
    ) -> Self {
        let mut anim = Self::new(animation_type, duration, ease_type);
        anim.loop_count = None;
        anim
    }

    /// 设置是否在循环时反向播放
    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse_on_loop = reverse;
        self
    }

    /// 获取当前动画进度 (0.0 到 1.0)
    pub fn progress(&self) -> f32 {
        if !self.is_active {
            return 1.0;
        }

        let elapsed = self.start_time.elapsed();
        let raw_progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        
        // 处理循环
        let loop_progress = raw_progress % 1.0;
        let loops_completed = raw_progress as u32;

        // 检查是否应该停止
        if let Some(max_loops) = self.loop_count {
            if loops_completed >= max_loops {
                return 1.0;
            }
        }

        // 处理反向播放
        let final_progress = if self.reverse_on_loop && loops_completed % 2 == 1 {
            1.0 - loop_progress
        } else {
            loop_progress
        };

        self.ease_type.apply(final_progress.clamp(0.0, 1.0))
    }

    /// 检查动画是否完成
    pub fn is_finished(&self) -> bool {
        if !self.is_active {
            return true;
        }

        if let Some(max_loops) = self.loop_count {
            let elapsed = self.start_time.elapsed();
            let total_progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
            total_progress >= max_loops as f32
        } else {
            false // 无限循环永不结束
        }
    }

    /// 重新开始动画
    pub fn restart(&mut self) {
        self.start_time = Instant::now();
        self.current_loop = 0;
        self.is_active = true;
    }

    /// 停止动画
    pub fn stop(&mut self) {
        self.is_active = false;
    }
}

/// 动画效果值
#[derive(Debug, Clone)]
pub struct AnimationValue {
    pub alpha: f32,        // 透明度 (0.0 到 1.0)
    pub offset_x: f32,     // X偏移
    pub offset_y: f32,     // Y偏移
    pub scale_x: f32,      // X缩放
    pub scale_y: f32,      // Y缩放
    pub color_intensity: f32, // 颜色强度
}

impl Default for AnimationValue {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            color_intensity: 1.0,
        }
    }
}

impl Animation {
    /// 计算当前动画值
    pub fn value(&self) -> AnimationValue {
        let progress = self.progress();
        let mut value = AnimationValue::default();

        match self.animation_type {
            AnimationType::FadeIn => {
                value.alpha = progress;
            }
            AnimationType::FadeOut => {
                value.alpha = 1.0 - progress;
            }
            AnimationType::SlideUp => {
                value.offset_y = (1.0 - progress) * -10.0;
                value.alpha = progress;
            }
            AnimationType::SlideDown => {
                value.offset_y = (1.0 - progress) * 10.0;
                value.alpha = progress;
            }
            AnimationType::SlideLeft => {
                value.offset_x = (1.0 - progress) * -20.0;
                value.alpha = progress;
            }
            AnimationType::SlideRight => {
                value.offset_x = (1.0 - progress) * 20.0;
                value.alpha = progress;
            }
            AnimationType::Pulse => {
                let pulse = (progress * 2.0 * std::f32::consts::PI).sin();
                value.color_intensity = 0.7 + 0.3 * pulse.abs();
            }
            AnimationType::Flash => {
                value.color_intensity = if (progress * 10.0) as u32 % 2 == 0 { 1.0 } else { 0.3 };
            }
            AnimationType::Shake => {
                let shake = (progress * 20.0 * std::f32::consts::PI).sin();
                value.offset_x = shake * 2.0;
            }
            AnimationType::Bounce => {
                let bounce = (progress * std::f32::consts::PI).sin().abs();
                value.offset_y = -bounce * 5.0;
            }
            AnimationType::Grow => {
                value.scale_x = progress;
                value.scale_y = progress;
                value.alpha = progress;
            }
            AnimationType::Shrink => {
                value.scale_x = 1.0 - progress;
                value.scale_y = 1.0 - progress;
                value.alpha = 1.0 - progress;
            }
            AnimationType::Typewriter => {
                // 打字机效果通过字符数控制，在渲染时处理
                value.alpha = 1.0;
            }
        }

        value
    }
}

/// 动画管理器
pub struct AnimationManager {
    animations: HashMap<String, Animation>,
    global_speed: f32, // 全局动画速度倍数
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            global_speed: 1.0,
        }
    }

    /// 添加动画
    pub fn add_animation(&mut self, id: String, animation: Animation) {
        self.animations.insert(id, animation);
    }

    /// 移除动画
    pub fn remove_animation(&mut self, id: &str) {
        self.animations.remove(id);
    }

    /// 获取动画值
    pub fn get_value(&self, id: &str) -> Option<AnimationValue> {
        self.animations.get(id).map(|anim| anim.value())
    }

    /// 更新所有动画（移除已完成的）
    pub fn update(&mut self) {
        self.animations.retain(|_id, anim| !anim.is_finished());
    }

    /// 清空所有动画
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// 设置全局动画速度
    pub fn set_global_speed(&mut self, speed: f32) {
        self.global_speed = speed.max(0.1);
    }

    /// 暂停/恢复所有动画
    pub fn pause_all(&mut self) {
        for animation in self.animations.values_mut() {
            animation.stop();
        }
    }

    pub fn resume_all(&mut self) {
        for animation in self.animations.values_mut() {
            animation.is_active = true;
        }
    }

    /// 重新开始所有动画
    pub fn restart_all(&mut self) {
        for animation in self.animations.values_mut() {
            animation.restart();
        }
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 预设动画创建器
pub struct AnimationPresets;

impl AnimationPresets {
    /// 菜单淡入
    pub fn menu_fade_in() -> Animation {
        Animation::new(
            AnimationType::FadeIn,
            Duration::from_millis(300),
            EaseType::EaseOut,
        )
    }

    /// 菜单淡出
    pub fn menu_fade_out() -> Animation {
        Animation::new(
            AnimationType::FadeOut,
            Duration::from_millis(200),
            EaseType::EaseIn,
        )
    }

    /// 游戏界面滑入
    pub fn game_slide_in() -> Animation {
        Animation::new(
            AnimationType::SlideUp,
            Duration::from_millis(400),
            EaseType::EaseOut,
        )
    }

    /// 按钮脉冲效果
    pub fn button_pulse() -> Animation {
        Animation::infinite(
            AnimationType::Pulse,
            Duration::from_millis(1000),
            EaseType::EaseInOut,
        )
    }

    /// 战斗闪烁
    pub fn combat_flash() -> Animation {
        Animation::looped(
            AnimationType::Flash,
            Duration::from_millis(200),
            EaseType::Linear,
            3,
        )
    }

    /// 错误震动
    pub fn error_shake() -> Animation {
        Animation::looped(
            AnimationType::Shake,
            Duration::from_millis(100),
            EaseType::Linear,
            5,
        )
    }

    /// 物品弹跳
    pub fn item_bounce() -> Animation {
        Animation::new(
            AnimationType::Bounce,
            Duration::from_millis(600),
            EaseType::Bounce,
        )
    }

    /// 消息打字机
    pub fn message_typewriter() -> Animation {
        Animation::new(
            AnimationType::Typewriter,
            Duration::from_millis(50), // 每字符50ms
            EaseType::Linear,
        )
    }
}

/// 动画辅助函数
pub mod animation_helpers {
    use super::*;
    use ratatui::style::{Color, Style};

    /// 应用透明度到颜色
    pub fn apply_alpha(color: Color, alpha: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let _a = (alpha * 255.0) as u8;
                Color::Rgb(
                    ((r as f32 * alpha) as u8).min(r),
                    ((g as f32 * alpha) as u8).min(g),
                    ((b as f32 * alpha) as u8).min(b),
                )
            }
            _ => color, // 对于非RGB颜色，保持不变
        }
    }

    /// 应用颜色强度
    pub fn apply_intensity(color: Color, intensity: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                ((r as f32 * intensity) as u8).min(255),
                ((g as f32 * intensity) as u8).min(255),
                ((b as f32 * intensity) as u8).min(255),
            ),
            _ => color,
        }
    }

    /// 应用动画值到样式
    pub fn apply_animation_to_style(style: Style, animation_value: &AnimationValue) -> Style {
        let mut new_style = style;

        // 应用透明度和颜色强度
        if let Some(fg) = style.fg {
            let fg_with_alpha = apply_alpha(fg, animation_value.alpha);
            let fg_with_intensity = apply_intensity(fg_with_alpha, animation_value.color_intensity);
            new_style = new_style.fg(fg_with_intensity);
        }

        if let Some(bg) = style.bg {
            let bg_with_alpha = apply_alpha(bg, animation_value.alpha);
            let bg_with_intensity = apply_intensity(bg_with_alpha, animation_value.color_intensity);
            new_style = new_style.bg(bg_with_intensity);
        }

        new_style
    }

    /// 为打字机效果截取文本
    pub fn typewriter_text(text: &str, progress: f32) -> String {
        let total_chars = text.len();
        let visible_chars = (total_chars as f32 * progress) as usize;
        text.chars().take(visible_chars).collect()
    }
}