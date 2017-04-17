use effect::{EffectManager, DamageDependency};
use item::Item;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Attribute {
  Agility,
  Intelligence,
  Strength,
}

#[derive(Debug, Clone)]
pub enum AttackCapability {
  Melee,
  Ranged,
}

// TODO: where to put illusions. Have create_illusion(dmgdealt, takne) method on hero?
#[derive(Debug, Clone)]
pub struct Hero {
  pub name: String,

  pub level: usize,

  pub effects: EffectManager,

  pub primary_attribute: Attribute,
  pub base_agility: f64,
  pub agility_gain: f64,
  pub base_intelligence: f64,
  pub intelligence_gain: f64,
  pub base_strength: f64,
  pub strength_gain: f64,

  pub starting_damage_min: f64,
  pub starting_damage_max: f64,
  pub base_attack_time: f64,
  // pub attack_posize: f64,
  // pub missile_speed: Option<f64>,
  pub attack_capability: AttackCapability,

  pub base_hp: f64,
  pub base_hp_regeneration: f64,
  pub base_mana: f64,
  pub base_mana_regeneration: f64,

  pub base_move_speed: f64,
  // pub turn_rate: f64,
  //
  // pub sight_range_day: f64,
  // pub sight_range_night: f64,
  pub base_armor: f64,
  pub base_magic_amplification: f64,
}

// Functions prefixed with "hero" do not take effects sizeo account
impl Hero {
  pub fn new() -> Hero {
    Hero {
      name: "Unnamed".to_string(),
      level: 1,
      effects: EffectManager::new(),
      primary_attribute: Attribute::Agility,
      base_agility: 0.0,
      agility_gain: 0.0,
      base_intelligence: 0.0,
      intelligence_gain: 0.0,
      base_strength: 0.0,
      strength_gain: 0.0,
      starting_damage_min: 0.0,
      starting_damage_max: 0.0,
      base_attack_time: 1.7,
      // attack_posize: 0.0, missile_speed: None,
      attack_capability: AttackCapability::Ranged,
      base_hp: 150.0,
      base_hp_regeneration: 0.25,
      base_mana: 0.0,
      base_mana_regeneration: 0.01,
      base_move_speed: 0.0,
      // turn_rate: 0.0, sight_range_day: 0.0, sight_range_night: 0.0,
      base_armor: 0.0,
      base_magic_amplification: 0.75,
    }
  }
  pub fn hero_agility(&self) -> f64 {
    self.base_agility + self.agility_gain * (self.level - 1) as f64
  }
  pub fn agility(&self) -> f64 {
    self.hero_agility() + self.effects.agility
  }
  pub fn hero_intelligence(&self) -> f64 {
    self.base_intelligence + self.intelligence_gain * (self.level - 1) as f64
  }
  pub fn intelligence(&self) -> f64 {
    self.hero_intelligence() + self.effects.intelligence
  }
  pub fn hero_strength(&self) -> f64 {
    self.base_strength + self.strength_gain * (self.level - 1) as f64
  }
  pub fn strength(&self) -> f64 {
    self.hero_strength() + self.effects.strength
  }
  pub fn average_starting_damage(&self) -> f64 {
    (self.starting_damage_min + self.starting_damage_max) / 2.0
  }
  pub fn primary_attribute_damage(&self) -> f64 {
    match self.primary_attribute {
      Attribute::Agility => self.hero_agility() + self.effects.agility,
      Attribute::Intelligence => self.hero_intelligence() + self.effects.intelligence,
      Attribute::Strength => self.hero_strength() + self.effects.strength,
    }
  }
  pub fn base_damage(&self) -> f64 {
    self.primary_attribute_damage() + self.average_starting_damage()
  }
  pub fn dependency_damage(&self, deps: &Vec<(DamageDependency, f64)>) -> f64 {
    deps.iter().fold(0.0, |acc, &(ref dep, amount)| {
      acc +
      amount *
      match dep {
        &DamageDependency::Agility => self.agility(),
        &DamageDependency::Intelligence => self.intelligence(),
        &DamageDependency::Strength => self.strength(),
        &DamageDependency::HP => self.hp(),
        &DamageDependency::Mana => self.mana(),
        &DamageDependency::BaseDamage => self.base_damage(),
      }
    })
  }
  pub fn damage_per_hit_physical(&self) -> f64 {
    let dependency_damage_crittable = self.dependency_damage(&self.effects.dependency_as_attack_damage);
    let damage_extra_physical = self.effects.extra_damage_physical +
                                self.dependency_damage(&self.effects.dependency_as_extra_damage_physical);
    let crittable_damage = self.base_damage() + self.effects.attack_damage + dependency_damage_crittable;
    let amplifyable_damage = crittable_damage * self.effects.critical_strike_average + damage_extra_physical;
    amplifyable_damage * self.effects.amplify_damage_dealt
  }
  pub fn damage_per_hit_magical(&self) -> f64 {
    let amplifyable_damage = self.effects.extra_damage_magical + self.dependency_damage(&self.effects.dependency_as_extra_damage_magical);
    amplifyable_damage * self.effects.amplify_damage_dealt
  }
  pub fn damage_per_hit_pure(&self) -> f64 {
    let amplifyable_damage = self.effects.extra_damage_pure + self.dependency_damage(&self.effects.dependency_as_extra_damage_pure);
    amplifyable_damage * self.effects.amplify_damage_dealt
  }
  pub fn attack_speed(&self) -> f64 {
    match 100.0 + self.agility() + self.effects.attack_speed {
      x if x < 20.0 => 20.0,
      x if x > 600.0 => 600.0,
      x => x,
    }
  }
  pub fn attacks_per_second(&self) -> f64 {
    (self.attack_speed() / 100.0) / self.base_attack_time
  }
  pub fn damage_per_second_physical(&self) -> f64 {
    self.damage_per_hit_physical() * self.attacks_per_second()
  }
  pub fn hp_regeneration(&self) -> f64 {
    let absolute = self.base_hp_regeneration + self.strength() * 0.03 + self.effects.hp_regeneration_absolute;
    let relative = self.hp() * self.effects.hp_regeneration_relative;
    absolute + relative
  }
  pub fn hp(&self) -> f64 {
    let raw_hp = self.base_hp + self.effects.hp;
    raw_hp + self.strength() * 19.0
  }
  pub fn mana_regeneration(&self) -> f64 {
    let base_absolute = self.base_mana_regeneration + self.intelligence() * 0.04;
    let other_absolute = self.effects.mana_regeneration_absolute;
    let relative = self.effects.mana_regeneration_relative;
    base_absolute * relative + other_absolute
  }
  pub fn mana(&self) -> f64 {
    let raw_mana = self.base_mana + self.effects.mana;
    raw_mana + self.intelligence() * 13.0
  }
  pub fn hero_armor(&self) -> f64 {
    self.base_armor + self.agility() * 0.14
  }
  pub fn armor(&self) -> f64 {
    self.hero_armor() + self.effects.armor
  }
  pub fn armor_amplification(&self) -> f64 {
    // expressed as the factor incoming physical attack will be multiplied with to get the damage dealt
    let armor = self.armor();
    match armor {
      x if x > 0.0 => 1.0 - (0.06 * armor) / (1.0 + 0.06 * armor),
      x if x < 0.0 => 1.0 + (0.06 * armor.abs()) / (1.0 + 0.06 * armor.abs()),
      _ => 1.0,
    }
  }
  pub fn magic_amplification(&self) -> f64 {
    self.base_magic_amplification * self.effects.amplify_magical_damage_taken
  }
  pub fn effective_hp_physical(&self) -> f64 {
    let armor_factor = 1.0 / self.armor_amplification();
    let evasion_factor = 1.0 / (1.0 - self.effects.evasion_average);
    self.hp() * armor_factor * evasion_factor / self.effects.amplify_damage_taken
  }
  // Returns by how much the heroes hp would decrease
  pub fn take_damage_physical(&self, damage: f64) -> f64 {
    let block_amount = match self.attack_capability {
      AttackCapability::Melee => self.effects.damage_block_average_melee,
      AttackCapability::Ranged => self.effects.damage_block_average_ranged,
    };
    let damage_after_block = match damage - block_amount {
      x if x < 0.0 => 0.0,
      x => x,
    };
    damage_after_block * self.armor_amplification() * (1.0 - self.effects.evasion_average) * self.effects.amplify_damage_taken
  }
  pub fn take_damage_magical(&self, damage: f64) -> f64 {
    damage * self.magic_amplification() * self.effects.amplify_damage_taken
  }
  pub fn take_damage_pure(&self, damage: f64) -> f64 {
    damage * self.effects.amplify_damage_taken
  }
  pub fn effective_hp_magical(&self) -> f64 {
    self.hp() / self.magic_amplification()
  }
  pub fn move_speed(&self) -> f64 {
    // TODO: check for non stacking ms from boots
    let absolute = self.base_move_speed + self.effects.move_speed_absolute;
    let relative = self.effects.move_speed_relative;
    absolute * relative
  }
  pub fn add_item(&mut self, item: &Item) {
    for effect in item.effects.iter() {
      self.effects.add_effect(effect)
    }
  }
  pub fn add_items(&mut self, items: &Vec<&Item>) {
    for item in items.iter() {
      self.add_item(*item);
    }
  }
  pub fn time_to_kill(attacker: &Hero, other: &Hero) -> f64 {
    let magical = other.take_damage_magical(attacker.damage_per_hit_magical());
    let physical = other.take_damage_physical(attacker.damage_per_hit_physical());
    let pured = other.take_damage_pure(attacker.damage_per_hit_pure());
    other.hp() / ((magical + physical + pured) * attacker.attacks_per_second())
  }
}
