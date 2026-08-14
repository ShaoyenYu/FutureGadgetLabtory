use crate::save_data::PersistentSaveData;
use soulforge_combat::components::CombatStats;
use soulforge_data::loader::GameDatabase;

/// 局外天赋加成汇总结构
#[derive(Debug, Clone, Default)]
pub struct MetaTalentBonuses {
    pub bonus_hp: f32,
    pub bonus_damage: f32,
    pub bonus_speed: f32,
    pub bonus_armor: f32,
}

impl PersistentSaveData {
    /// 计算所有已升级天赋提供的累计被动属性加成
    pub fn calculate_bonuses(&self, db: &GameDatabase) -> MetaTalentBonuses {
        let mut bonuses = MetaTalentBonuses::default();

        for talent in &db.talents.talents {
            let level = self.talent_levels.get(&talent.id).copied().unwrap_or(0);
            let total_val = (level as f32) * talent.value_per_level;

            match talent.stat_type.as_str() {
                "MaxHp" => bonuses.bonus_hp += total_val,
                "Damage" => bonuses.bonus_damage += total_val,
                "MoveSpeed" => bonuses.bonus_speed += total_val,
                "Armor" => bonuses.bonus_armor += total_val,
                _ => {}
            }
        }

        bonuses
    }

    /// 升级天赋接口
    pub fn try_upgrade_talent(&mut self, talent_id: &str, db: &GameDatabase) -> bool {
        let Some(talent) = db.talents.talents.iter().find(|t| t.id == talent_id) else {
            return false;
        };

        let current_level = self.talent_levels.get(talent_id).copied().unwrap_or(0);
        if current_level >= talent.max_level {
            return false;
        }

        let cost = (talent.cost_base as f32 * talent.cost_multiplier.powi(current_level as i32)) as u32;
        if self.gold >= cost {
            self.gold -= cost;
            self.talent_levels.insert(talent_id.to_string(), current_level + 1);
            self.save_to_disk();
            true
        } else {
            false
        }
    }
}

/// 局内初始化时应用天赋属性到主角面板
pub fn apply_talents_to_player(
    stats: &mut CombatStats,
    save_data: &PersistentSaveData,
    db: &GameDatabase,
) {
    let bonuses = save_data.calculate_bonuses(db);
    stats.max_hp += bonuses.bonus_hp;
    stats.current_hp = stats.max_hp;
    stats.base_damage += bonuses.bonus_damage;
    stats.move_speed += bonuses.bonus_speed;
    stats.armor += bonuses.bonus_armor;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talent_upgrade_and_bonus_application() {
        let db = GameDatabase::load_from_disk_or_default();
        let mut save = PersistentSaveData::default();
        save.gold = 500;

        assert!(save.try_upgrade_talent("talent_hp", &db));
        let bonuses = save.calculate_bonuses(&db);
        assert_eq!(bonuses.bonus_hp, 15.0);

        let mut stats = CombatStats::default();
        apply_talents_to_player(&mut stats, &save, &db);
        assert_eq!(stats.max_hp, 115.0);
    }
}
