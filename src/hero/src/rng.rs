// src/hero/rng.rs
use bincode::{Decode, Encode};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// 英雄专用的确定性RNG系统
#[derive(Debug, Clone)]
pub struct HeroRng {
    rng: Pcg32,
    seed: u64,
}

impl HeroRng {
    /// 使用随机种子创建新RNG
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg32::seed_from_u64(seed),
            seed,
        }
    }

    /// 获取当前种子值
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// 重置RNG状态（使用当前种子）
    pub fn reset(&mut self) {
        self.rng = Pcg32::seed_from_u64(self.seed);
    }

    /// 使用新种子重置RNG
    pub fn reseed(&mut self, new_seed: u64) {
        self.seed = new_seed;
        self.reset();
    }

    /// 生成随机布尔值
    pub fn gen_bool(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability)
    }

    /// 从列表中随机选择
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let idx = self.gen_range(0, items.len());
            Some(&items[idx])
        }
    }

    /// 从列表中随机选择可变引用
    pub fn choose_mut<'a, T>(&mut self, items: &'a mut [T]) -> Option<&'a mut T> {
        if items.is_empty() {
            None
        } else {
            let idx = self.gen_range(0, items.len());
            Some(&mut items[idx])
        }
    }

    /// 随机打乱切片
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.rng.shuffle(slice);
    }

    /// 计算带随机性的防御值（SPD风格）
    pub fn defense_roll(&mut self, base_defense: u32) -> u32 {
        let defense_factor = self.gen_range(0.7, 1.3);
        (base_defense as f32 * defense_factor) as u32
    }
    pub fn gen_range(&mut self, range: std::ops::Range<f32>) -> f32 {
        self.rng.gen_range(range)
    }
}

// 序列化实现
impl Serialize for HeroRng {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.seed)
    }
}

// 反序列化实现
impl<'de> Deserialize<'de> for HeroRng {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed = u64::deserialize(deserializer)?;
        Ok(Self::new(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_rng() {
        let mut rng1 = HeroRng::new(123);
        let mut rng2 = HeroRng::new(123);

        // 相同种子应产生相同序列
        assert_eq!(rng1.gen_range(0..100), rng2.gen_range(0..100));
        assert_eq!(rng1.gen_bool(0.5), rng2.gen_bool(0.5));

        // 重置后应恢复相同序列
        rng1.reseed(456);
        rng2.reseed(456);
        assert_eq!(rng1.gen_range(0..100), rng2.gen_range(0..100));
    }

    #[test]
    fn test_defense_roll() {
        let mut rng = HeroRng::new(789);
        let base_defense = 10;
        let roll = rng.defense_roll(base_defense);

        // 检查防御值在预期范围内 (7-13)
        assert!(roll >= 7);
        assert!(roll <= 13);
    }
}
