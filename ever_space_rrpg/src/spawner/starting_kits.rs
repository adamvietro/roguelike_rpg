use ron::de::from_reader;
use serde::Deserialize;
use std::fs::File;

/// One class's guaranteed starting inventory. Item names must exactly
/// match a `Template.name` in resources/template.ron - this bypasses the
/// normal ambient/prefab/battle-loot pools entirely (see
/// Templates::spawn_named_item), so it works even for `prefab_only` or
/// `levels: []` (battle-loot-only) items like a starting Staff or a
/// starting Fireball.
#[derive(Clone, Deserialize, Debug)]
pub struct StartingKit {
    pub class: String,
    pub items: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct StartingKits {
    pub kits: Vec<StartingKit>,
}

impl StartingKits {
    pub fn load() -> Self {
        let file = File::open("resources/starting_kits.ron").expect("Failed opening file");
        from_reader(file).expect("Unable to load starting kits")
    }

    /// The item names `class` should start with, or an empty slice if no
    /// kit is defined for it - such a class just starts with nothing
    /// extra, same as every class did before this system existed.
    pub fn items_for(&self, class: &str) -> &[String] {
        self.kits
            .iter()
            .find(|kit| kit.class == class)
            .map(|kit| kit.items.as_slice())
            .unwrap_or(&[])
    }
}
