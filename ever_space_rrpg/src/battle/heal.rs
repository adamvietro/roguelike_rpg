use crate::prelude::*;

// --- Heal ------------------------------------------------------------------
//
// Restores HP to the player right now. Used by TechniqueEffect::Heal -
// not used by any current Barbarian technique, included so a Mage
// healing spell (or any future class's) has somewhere to plug in without
// another battle.rs change.

/// TechniqueEffect::Heal - restores `amount` HP to the player, clamped to
/// max.
pub fn apply(ecs: &mut World, battle: &mut Battle, amount: i32) -> String {
    heal_entity(ecs, battle.player, amount);
    battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
    format!("Heal {} HP.", amount)
}
