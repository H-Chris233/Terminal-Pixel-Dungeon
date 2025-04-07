use rand::Rng;
use bincode::{Decode, Encode};
use serde::{Serialize, Deserialize};

/// 武器附魔效果（参考破碎的像素地牢附魔系统）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum WeaponEnhance {
    // 常规附魔
    Burning,      // 燃烧：攻击有概率点燃敌人
    Stunning,     // 击晕：攻击有概率眩晕敌人
    Vampiric,     // 吸血：攻击恢复生命值
    Lucky,        // 幸运：增加暴击率
    // 诅咒附魔
    Cursed,       // 诅咒：随机负面效果
    Fragile,      // 易碎：使用次数有限
}

/// 武器改造方向（参考武器改造系统）
#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub enum WeaponMod {
    Damage,       // 偏向伤害（增加伤害但降低攻速）
    Speed,        // 偏向速度（增加攻速但降低伤害）
    Balanced,     // 平衡改造
}

/// 武器数据结构（参考破碎的像素地牢武器属性）
#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,       //名字
    pub tier: usize,          // 品阶1-5
    pub damage: (i32, i32),    // 伤害范围(min,max)
    pub hit_chance: f32,       // 命中率(0.0-1.0)
    pub str_requirement: i32,  // 力量需求
    pub enchanted: Option<WeaponEnhance>, // 附魔效果
    pub modifier: WeaponMod,   // 改造方向
    pub upgrade_level: i32,    // 强化等级（+1,+2等）
    pub cursed: bool,         // 是否被诅咒
    pub durability: Option<i32>, // 耐久度（某些特殊武器）
}

impl Weapon {
    /// 创建新武器（根据品阶初始化基础属性）
    pub fn new(tier: usize) -> Self {
        let (damage, str_req) = match tier {
            1 => ((1, 4), 10),  // 一阶武器
            2 => ((3, 8), 13),  // 二阶武器
            3 => ((6, 12), 16), // 三阶武器
            4 => ((10, 20), 19),// 四阶武器
            5 => ((15, 30), 22),// 五阶武器
            _ => panic!("Invalid weapon tier"),
        };

        Self {
            tier,
            damage,
            hit_chance: 0.8,
            str_requirement: str_req,
            enchanted: None,
            modifier: WeaponMod::Balanced,
            upgrade_level: 0,
            cursed: false,
            durability: None,
        }
    }

    /// 强化武器（降低力量需求并提高伤害）
    pub fn upgrade(&mut self) {
        self.upgrade_level += 1;
        self.str_requirement = (self.str_requirement - 1).max(1); // 每+3再降1点
        
        // 伤害成长公式：每级最小伤害+1，最大伤害+2
        self.damage.0 += 1;
        self.damage.1 += 2;
    }

    /// 计算实际伤害（考虑力量加成和诅咒惩罚）
    pub fn calculate_damage(&self, user_str: i32) -> i32 {
        let str_diff = user_str - self.str_requirement;
        let mut damage = rand::rng().random_range(self.damage.0..=self.damage.1);

        // 力量修正
        if str_diff > 0 {
            damage += str_diff; // 力量超过需求时额外伤害
        } else if str_diff < 0 {
            damage = (damage as f32 * 0.7f32.powi(-str_diff)) as i32; // 力量不足时惩罚
        }

        // 诅咒惩罚
        if self.cursed {
            damage = (damage as f32 * 0.6) as i32;
        }

        damage.max(1)
    }

    /// 添加随机附魔（参考附魔获取方式）
    pub fn add_random_enhancement(&mut self) {
        let enhancements = vec![
            WeaponEnhance::Burning,
            WeaponEnhance::Stunning,
            WeaponEnhance::Vampiric,
            WeaponEnhance::Lucky,
        ];
        
        self.enchanted = Some(enhancements[rand::rng().random_range(0..enhancements.len())].clone());
    }

    /// 武器鉴定逻辑（参考破碎的像素地牢鉴定方法）
    pub fn identify(&mut self) {
        self.cursed = match self.enchanted {
            Some(WeaponEnhance::Cursed) | Some(WeaponEnhance::Fragile) => true,
            _ => false,
        };
    }

    /// 改造武器（参考武器改造系统）
    pub fn modify(&mut self, direction: WeaponMod) {
        self.modifier = direction;
        match direction {
            WeaponMod::Damage => {
                self.damage.0 += 2;
                self.damage.1 += 3;
                self.hit_chance = (self.hit_chance * 0.9).max(0.5);
            },
            WeaponMod::Speed => {
                self.hit_chance = (self.hit_chance * 1.1).min(0.95);
                self.damage.0 = (self.damage.0 as f32 * 0.8) as i32;
                self.damage.1 = (self.damage.1 as f32 * 0.8) as i32;
            },
            WeaponMod::Balanced => {
                self.damage.0 += 1;
                self.damage.1 += 1;
            }
        }
    }
}

/// 武器类型枚举（参考近战武器分类）
#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,       // 剑类：平衡型
    Dagger,      // 匕首：高速低伤
    Greataxe,    // 巨斧：低速高伤
    Spear,       // 长矛：中距攻击
    // 其他特殊类型...
}
