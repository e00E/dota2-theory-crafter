// Parse json files generated from the dota2 files by parse_heroes

use rustc_serialize::json::Json;
use std::path::Path;
use std::fs;
use std::io::Read;
use std::convert::AsRef;
use hero::{Hero, Attribute, AttackCapability};
use effect::{Effect, ExtraDamage, DamageDependency};
use item::Item;

// TODO: use base_hero correctly
// TODO: error handling


// Parse all json files as heroes in the specified directory recursively (including subdirectories).
pub fn parse_all_heroes(path: &str) -> Vec<Hero> {
  let mut heroes = Vec::new();
  for path in fs::read_dir(&Path::new(path))
    .unwrap()
    .filter_map(|path| {
      match path {
        Ok(path) => {
          let path = path.path();
          if path.is_file() { Some(path) } else { None }
        }
        Err(..) => None,
      }
    }) {
    heroes.push(parse_single_hero(path.as_path()));
  }
  heroes
}

// Parse all json files as items in the specified directory recursively (including subdirectories).
pub fn parse_all_items(path: &str) -> Vec<Item> {
  let mut items = Vec::new();
  for path in fs::read_dir(&Path::new(path))
    .unwrap()
    .filter_map(|path| {
      match path {
        Ok(path) => {
          let path = path.path();
          if path.is_file() { Some(path) } else { None }
        }
        Err(..) => None,
      }
    }) {
    items.push(parse_single_item(path.as_path()));
  }
  items
}

pub fn parse_single_hero(path: &Path) -> Hero {
  let mut string = String::new();
  fs::File::open(&path).unwrap().read_to_string(&mut string).unwrap();
  let json = Json::from_str(string.as_ref()).unwrap();
  let object = json.as_object().unwrap();

  let mut hero = Hero::new();
  // Some keys are always present while some default to the values in Base. For the latter we use the default values in the Hero class.
  hero.name = object["Name"].as_string().unwrap().to_string();
  hero.primary_attribute = match object["AttributePrimary"].as_string().unwrap() {
    "DOTA_ATTRIBUTE_AGILITY" => Attribute::Agility,
    "DOTA_ATTRIBUTE_INTELLECT" => Attribute::Intelligence,
    "DOTA_ATTRIBUTE_STRENGTH" => Attribute::Strength,
    _ => panic!(),
  };
  hero.attack_capability = match object["AttackCapabilities"].as_string().unwrap() {
    "DOTA_UNIT_CAP_MELEE_ATTACK" => AttackCapability::Melee,
    "DOTA_UNIT_CAP_RANGED_ATTACK" => AttackCapability::Ranged,
    _ => panic!(),
  };
  hero.base_agility = object["AttributeBaseAgility"].as_f64().unwrap();
  hero.agility_gain = object["AttributeAgilityGain"].as_f64().unwrap();
  hero.base_intelligence = object["AttributeBaseIntelligence"].as_f64().unwrap();
  hero.intelligence_gain = object["AttributeIntelligenceGain"].as_f64().unwrap();
  hero.base_strength = object["AttributeBaseStrength"].as_f64().unwrap();
  hero.strength_gain = object["AttributeStrengthGain"].as_f64().unwrap();
  hero.starting_damage_min = object["AttackDamageMin"].as_f64().unwrap();
  hero.starting_damage_max = object["AttackDamageMax"].as_f64().unwrap();
  hero.base_attack_time = object["AttackRate"].as_f64().unwrap();
  if object.contains_key(&"StatusHealth".to_string()) {
    hero.base_hp = object["StatusHealth"].as_f64().unwrap();
  }
  if object.contains_key(&"StatusHealthRegen".to_string()) {
    hero.base_hp_regeneration = object["StatusHealthRegen"].as_f64().unwrap();
  }
  if object.contains_key(&"StatusMana".to_string()) {
    hero.base_mana = object["StatusMana"].as_f64().unwrap();
  }
  if object.contains_key(&"StatusManaRegen".to_string()) {
    hero.base_mana_regeneration = object["StatusManaRegen"].as_f64().unwrap();
  }
  hero.base_move_speed = object["MovementSpeed"].as_f64().unwrap();
  hero.base_armor = object["ArmorPhysical"].as_f64().unwrap();
  if object.contains_key(&"MagicalResistance".to_string()) {
    hero.base_magic_amplification = 1.0 - object["MagicalResistance"].as_f64().unwrap() / 100.0;
  }

  hero
}

// TODO: How to parse items that are different on melee and range like Basher and Damage Block. For now add those manually somewhere else.
pub fn parse_single_item(path: &Path) -> Item {
  // TODO: clean up closure use here. It seems a bit wrong.
  let mut string = String::new();
  fs::File::open(&path).unwrap().read_to_string(&mut string).unwrap();
  let json = Json::from_str(string.as_ref()).unwrap();
  let object = json.as_object().unwrap();

  let mut item = Item::new();
  item.name = object["Name"].as_string().unwrap().to_string();
  item.cost = object["ItemCost"].as_f64().unwrap();

  // get the corresponding f64 to a key
  let get_f64 = |key: &str| -> f64 { object.get(&key.to_string()).unwrap().as_f64().unwrap() };

  // Checks if all keys are mapped
  let contains_all = |keys: &[&str]| -> bool {
    for k in keys.iter() {
      if !object.contains_key(&k.to_string()) {
        return false;
      }
    }
    true
  };
  // Consumes a value if its key exists.
  let try_consume = |key: &str, consume: &mut FnMut(f64)| {
    match object.get(&key.to_string()) {
      Some(ref json) => consume(json.as_f64().unwrap()),
      None => (),
    };
  };
  {
    // Same as above but consume creates an Effect that gets pushed onto the item.
    // This is in a separate scope because the Closure captures item.effects.effects until it goes out of scope.
    let mut try_consume_push = |key: &str, consume: &Fn(f64) -> Effect| {
      try_consume(key, &mut |value| item.effects.push(consume(value)));
    };
    // Keys that can get mapped to Effects on their own
    try_consume_push("bonus_damage", &|value| Effect::AttackDamage(value));
    try_consume_push("damage_aura",
                     &|value| Effect::DependencyAsAttackDamage(DamageDependency::BaseDamage, value / 100.0));
    try_consume_push("bonus_armor", &|value| Effect::Armor(value));
    try_consume_push("armor_aura", &|value| Effect::Armor(value));
    try_consume_push("aura_bonus_armor", &|value| Effect::Armor(value));
    try_consume_push("aura_positive_armor", &|value| Effect::Armor(value));
    try_consume_push("aura_armor", &|value| Effect::Armor(value));
    try_consume_push("bonus_health_regen",
                     &|value| Effect::HPRegenerationAbsolute(value));
    try_consume_push("bonus_regen",
                     &|value| Effect::HPRegenerationAbsolute(value));
    try_consume_push("aura_health_regen",
                     &|value| Effect::HPRegenerationAbsolute(value));
    try_consume_push("hp_regen", &|value| Effect::HPRegenerationAbsolute(value));
    try_consume_push("health_regen",
                     &|value| Effect::HPRegenerationAbsolute(value));
    try_consume_push("health_regen_rate",
                     &|value| Effect::HPRegenerationRelative(value / 100.0));
    try_consume_push("mana_regen_aura",
                     &|value| Effect::ManaRegenerationAbsolute(value));
    try_consume_push("aura_mana_regen",
                     &|value| Effect::ManaRegenerationAbsolute(value));
    try_consume_push("bonus_mana_regen",
                     &|value| Effect::ManaRegenerationRelative(value / 100.0));
    try_consume_push("mana_regen",
                     &|value| Effect::ManaRegenerationRelative(value / 100.0));
    try_consume_push("bonus_mana_regen_pct",
                     &|value| Effect::ManaRegenerationRelative(value / 100.0));
    try_consume_push("bonus_health", &|value| Effect::HP(value));
    try_consume_push("bonus_mana", &|value| Effect::Mana(value));
    try_consume_push("bonus_attack_speed", &|value| Effect::AttackSpeed(value));
    try_consume_push("bonus_speed", &|value| Effect::AttackSpeed(value));
    try_consume_push("aura_attack_speed", &|value| Effect::AttackSpeed(value));
    try_consume_push("bonus_aura_attack_speed_pct",
                     &|value| Effect::AttackSpeed(value));
    try_consume_push("bonus_evasion", &|value| Effect::Evasion(value / 100.0));
    try_consume_push("bonus_agility", &|value| Effect::Agility(value));
    try_consume_push("bonus_intellect", &|value| Effect::Intelligence(value));
    try_consume_push("bonus_intelligence", &|value| Effect::Intelligence(value));
    try_consume_push("bonus_strength", &|value| Effect::Strength(value));
    try_consume_push("bonus_spell_resist",
                     &|value| Effect::AmplifyMagicalDamageTaken(1.0 - value / 100.0));
    try_consume_push("magic_resistance",
                     &|value| Effect::AmplifyMagicalDamageTaken(1.0 - value / 100.0));
    try_consume_push("bonus_magical_armor",
                     &|value| Effect::AmplifyMagicalDamageTaken(1.0 - value / 100.0));
    try_consume_push("bonus_movement_speed",
                     &|value| Effect::MoveSpeedAbsolute(value));
    try_consume_push("bonus_movement", &|value| Effect::MoveSpeedAbsolute(value));
    try_consume_push("bonus_aura_movement_speed_pct",
                     &|value| Effect::MoveSpeedRelative(value / 100.0));
    try_consume_push("movement_speed_percent_bonus",
                     &|value| Effect::MoveSpeedRelative(value / 100.0));
  }
  // Keys that are a little bit harder to map
  // I would like to write the following, but it is not possible in rust atm due to a borrow checker bug:
  // try_get( "bonus_chance", |value1| try_get(
  // "bonus_chance_damage", |value2| item.effects.effects.push( Effect::ConditionalDamage(value1, value2, DamageType::Physical) ) ) );
  if contains_all(&["bonus_chance", "bonus_chance_damage"]) {
    let chance = get_f64("bonus_chance") / 100.0;
    let damage = get_f64("bonus_chance_damage");
    item.effects.push(Effect::ExtraDamage(ExtraDamage::Physical(damage * chance))); //Default to physical type
  }
  if contains_all(&["crit_chance", "crit_multiplier"]) {
    let chance = get_f64("crit_chance") / 100.0;
    let multiplier = get_f64("crit_multiplier") / 100.0;
    item.effects.push(Effect::CriticalStrike(chance, multiplier));
  }
  if contains_all(&["bash_chance", "bash_damage"]) {
    let chance = get_f64("bash_chance") / 100.0;
    let damage = get_f64("bash_damage");
    item.effects.push(Effect::ExtraDamage(ExtraDamage::Physical(damage * chance)));
  }
  if contains_all(&["bash_chance_melee", "bonus_chance_damage"]) {
    let chance = get_f64("bash_chance_melee") / 100.0;
    let damage = get_f64("bonus_chance_damage");
    item.effects.push(Effect::ExtraDamage(ExtraDamage::Physical(damage * chance)));
  }
  if contains_all(&["block_chance", "damage_block_melee", "damage_block_ranged"]) {
    let chance = get_f64("block_chance") / 100.0;
    let block_melee = get_f64("damage_block_melee");
    let block_ranged = get_f64("damage_block_ranged");
    item.effects.push(Effect::DamageBlock(chance, block_melee, block_ranged));
  }
  try_consume("bonus_all_stats",
              &mut |value| {
                item.effects.push(Effect::Agility(value));
                item.effects.push(Effect::Intelligence(value));
                item.effects.push(Effect::Strength(value));
              });
  try_consume("bonus_stats",
              &mut |value| {
                item.effects.push(Effect::Agility(value));
                item.effects.push(Effect::Intelligence(value));
                item.effects.push(Effect::Strength(value));
              });

  item
}
