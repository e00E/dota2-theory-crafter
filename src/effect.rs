use hero::AttackCapability;

#[derive(Debug, Clone)]
pub enum DamageDependency {
  Agility,
  Intelligence,
  Strength,
  HP,
  Mana,
  BaseDamage,
}

#[derive(Debug, Clone)]
pub enum ExtraDamage {
  Magical(f64),
  Physical(f64),
  Pure(f64),
}

#[derive(Debug, Clone)]
pub enum AttackModifier {
  ArmorReduction(f64),
  Lifesteal(f64),
  Truestrike,
}

// TODO: To implement non stacking items add an id field to an effect. MoveSpeedAbsolute(f64, string)
// for example and string can be boots, euls, ...
// should id be string? or int? Could have an item enum that can be converted to int: enum MoveSpeedItem{ Boots, Euls, ... }
// Or this could be checked when an item is added trough Hero::add_item. Then the id is only tied to an item and not effect, seems better.
// Hero would save which item types it arleady contains and an add_item of that kind simply does nothing.
#[derive(Debug, Clone)]
pub enum Effect {
  Agility(f64),
  Intelligence(f64),
  Strength(f64),

  AttackSpeed(f64), // 1 Agility yields 1 AttackSpeed, 1 HyperStone yields 55
  AttackDamage(f64),
  DependencyAsAttackDamage(DamageDependency, f64), // Like Drow Aura or Ursa ultimate
  DependencyAsExtraDamage(DamageDependency, ExtraDamage), // Like Riki backstab or Silencer Glaive
  ExtraDamage(ExtraDamage), // Like Mkb or Mjollnir procs
  CriticalStrike(f64, f64),
  AmplifyDamageDealt(f64), // multiplier by how much damage is increased. Can only be positive. Bloodrage would be 0.2
  // OutgoingMissChance(f64), //ratio. Tinker's Laser would be 0.8
  HP(f64),
  HPRegenerationAbsolute(f64),
  HPRegenerationRelative(f64), // ratio. Heart would be 0.05

  Mana(f64),
  ManaRegenerationAbsolute(f64),
  ManaRegenerationRelative(f64), // ratio. Sobi Mask would be 0.5

  Armor(f64),
  Evasion(f64), // ratio. Butterfly would be 0.3
  DamageBlock(f64, f64, f64), // chance, melee block amount, ranged block amount
  // multiplier by how much damage taken is increased/decreased. Bloodrage would be 0.2, BristleBack level 4 would be -0.4
  AmplifyDamageTaken(f64),
  // the factor incoming magic damage gets multiplied with to get the damage dealt.
  // Hood would be (1-0.3)=0.7 and Ethereal would be 1.4 . Has to be positive
  AmplifyMagicalDamageTaken(f64),
  MoveSpeedAbsolute(f64),
  MoveSpeedRelative(f64), // multiplier by which ms would be increased / decreased. S&Y would be 0.16
}

// Helper class that can compute various properties of multiple effects
// TODO: implement an ordering of items that decides in which order items have to be appliedd, as for example
// crits depend on the highest multiplier, etc
// TODO: use gettter instead of having the fields pub?
#[derive(Debug, Clone)]
pub struct EffectManager {
  pub agility: f64,
  pub intelligence: f64,
  pub strength: f64,
  pub attack_speed: f64,
  pub attack_damage: f64,
  // How far am I willing to unroll those enums..?
  pub dependency_as_attack_damage: Vec<(DamageDependency, f64)>,
  pub dependency_as_extra_damage_magical: Vec<(DamageDependency, f64)>,
  pub dependency_as_extra_damage_physical: Vec<(DamageDependency, f64)>,
  pub dependency_as_extra_damage_pure: Vec<(DamageDependency, f64)>,
  pub extra_damage_magical: f64,
  pub extra_damage_physical: f64,
  pub extra_damage_pure: f64,
  pub critical_strike: Vec<(f64, f64)>,
  // The multiplier outgoing damage can be multiplied to get the average damage dealt. Gets updated whenever another crit is added */
  pub critical_strike_average: f64,
  pub amplify_damage_dealt: f64, // multiplier by which outgoing damage will be multiplied
  pub hp: f64,
  pub hp_regeneration_absolute: f64,
  pub hp_regeneration_relative: f64,
  pub mana: f64,
  pub mana_regeneration_absolute: f64,
  pub mana_regeneration_relative: f64,
  pub armor: f64,
  pub evasion: Vec<f64>,
  pub evasion_average: f64, // average chance of an incoming attack to miss. Gets updated when a new evasion source is added
  pub damage_block: Vec<(f64, f64, f64)>,
  pub damage_block_average_melee: f64, // average damage blocked. Gets updated when new damage block source is added
  pub damage_block_average_ranged: f64,
  pub amplify_damage_taken: f64,
  pub amplify_magical_damage_taken: f64,
  pub move_speed_absolute: f64,
  pub move_speed_relative: f64,
}

// Can this maybe be implemented for Iterator<Item> to make it more general?
impl EffectManager {
  pub fn new() -> EffectManager {
    EffectManager {
      agility: 0.0,
      intelligence: 0.0,
      strength: 0.0,
      attack_speed: 0.0,
      attack_damage: 0.0,
      dependency_as_attack_damage: Vec::new(),
      dependency_as_extra_damage_magical: Vec::new(),
      dependency_as_extra_damage_physical: Vec::new(),
      dependency_as_extra_damage_pure: Vec::new(),
      extra_damage_magical: 0.0,
      extra_damage_physical: 0.0,
      extra_damage_pure: 0.0,
      critical_strike: Vec::new(),
      critical_strike_average: 1.0,
      amplify_damage_dealt: 1.0,
      hp: 0.0,
      hp_regeneration_absolute: 0.0,
      hp_regeneration_relative: 0.0,
      mana: 0.0,
      mana_regeneration_absolute: 0.0,
      mana_regeneration_relative: 1.0,
      armor: 0.0,
      evasion: Vec::new(),
      evasion_average: 0.0,
      damage_block: Vec::new(),
      damage_block_average_melee: 0.0,
      damage_block_average_ranged: 0.0,
      amplify_damage_taken: 1.0,
      amplify_magical_damage_taken: 1.0,
      move_speed_absolute: 0.0,
      move_speed_relative: 1.0,
    }
  }
  pub fn add_effect(&mut self, effect: &Effect) {
    match effect {
      &Effect::Agility(amount) => self.agility += amount,
      &Effect::Intelligence(amount) => self.intelligence += amount,
      &Effect::Strength(amount) => self.strength += amount,
      &Effect::AttackSpeed(amount) => self.attack_speed += amount,
      &Effect::AttackDamage(amount) => self.attack_damage += amount,
      &Effect::DependencyAsAttackDamage(ref dep, amount) => self.dependency_as_attack_damage.push((dep.clone(), amount)),
      &Effect::DependencyAsExtraDamage(ref dep, ExtraDamage::Magical(amount)) => {
        self.dependency_as_extra_damage_magical.push((dep.clone(), amount))
      }
      &Effect::DependencyAsExtraDamage(ref dep, ExtraDamage::Physical(amount)) => {
        self.dependency_as_extra_damage_physical.push((dep.clone(), amount))
      }
      &Effect::DependencyAsExtraDamage(ref dep, ExtraDamage::Pure(amount)) => {
        self.dependency_as_extra_damage_pure.push((dep.clone(), amount))
      }
      &Effect::ExtraDamage(ExtraDamage::Magical(amount)) => self.extra_damage_magical += amount,
      &Effect::ExtraDamage(ExtraDamage::Physical(amount)) => self.extra_damage_physical += amount,
      &Effect::ExtraDamage(ExtraDamage::Pure(amount)) => self.extra_damage_pure += amount,
      &Effect::CriticalStrike(chance, multiplier) => {
        self.critical_strike.push((chance, multiplier));
        self.update_critical_strike();
      }
      &Effect::AmplifyDamageDealt(amount) => self.amplify_damage_dealt += amount,
      &Effect::HP(amount) => self.hp += amount,
      &Effect::HPRegenerationAbsolute(amount) => self.hp_regeneration_absolute += amount,
      &Effect::HPRegenerationRelative(amount) => self.hp_regeneration_relative += amount,
      &Effect::Mana(amount) => self.mana += amount,
      &Effect::ManaRegenerationAbsolute(amount) => self.mana_regeneration_absolute += amount,
      &Effect::ManaRegenerationRelative(amount) => self.mana_regeneration_relative += amount,
      &Effect::Armor(amount) => self.armor += amount,
      &Effect::Evasion(probability) => {
        self.evasion.push(probability);
        self.update_evasion();
      }
      &Effect::DamageBlock(chance, melee, range) => {
        self.damage_block.push((chance, melee, range));
        self.update_damage_block();
      }
      &Effect::AmplifyDamageTaken(amount) => self.amplify_damage_taken += amount,
      &Effect::AmplifyMagicalDamageTaken(amount) => self.amplify_magical_damage_taken *= amount,
      &Effect::MoveSpeedAbsolute(amount) => self.move_speed_absolute += amount,
      &Effect::MoveSpeedRelative(amount) => self.move_speed_relative += amount,
    }
  }
  pub fn update_critical_strike(&mut self) {
    // In DotA2 if you have multiple critical strike sources, the highest multiplier will go first,
    // if it does not proc, then the 2nd highest goes, and so on...

    // Filter all critical strikes
    // sort the vector in reverse
    self.critical_strike.sort_by(|&(_, mult1), &(_, mult2)| {
      match mult2.partial_cmp(&mult1) {
        None => panic!("Critical Strike multiplier is NaN."),
        Some(ordering) => ordering,
      }
    });
    // compute the total damage factor
    self.critical_strike_average = 1.0;
    let mut probability = 1.0; //The probability that no critical strike occurred before
    for &(chance, multiplier) in self.critical_strike.iter() {
      self.critical_strike_average += chance * (multiplier - 1.0) * probability;
      probability *= 1.0 - chance;
    }
  }
  pub fn update_evasion(&mut self) {
    // Total evasion chance = [ 1 - (1 - first source of evasion) x (1 - second source of evasion) ... x (1 - n-source of evasion)]
    self.evasion_average = 1.0 - self.evasion.iter().fold(1.0, |acc, probability| acc * (1.0 - *probability));
  }
  pub fn update_damage_block(&mut self) {
    // In DotA2 if you have multiple damage block sources, the highest block amount will go first,
    // if it does not proc, then the 2nd highest goes, and so on...

    fn get_average(ac: AttackCapability, dbs: &Vec<(f64, f64, f64)>) -> f64 {
      let mut damage_blocks = Vec::new();
      for &(chance, melee, ranged) in dbs.iter() {
        damage_blocks.push((chance,
                            match ac {
                              AttackCapability::Melee => melee,
                              AttackCapability::Ranged => ranged,
                            }));
      }
      // sort the vector in reverse
      damage_blocks.sort_by(|&(_, block1), &(_, block2)| {
        match block2.partial_cmp(&block1) {
          None => panic!("Damage Block amount is NaN."),
          Some(ordering) => ordering,
        }
      });
      // compute the average damage blocked
      let mut result = 0.0;
      let mut probability = 1.0; //The probability that no damage block occurred before
      for &(chance, amount) in damage_blocks.iter() {
        result += chance * amount * probability;
        probability *= 1.0 - chance;
      }
      result
    }
    self.damage_block_average_melee = get_average(AttackCapability::Melee, &self.damage_block);
    self.damage_block_average_ranged = get_average(AttackCapability::Ranged, &self.damage_block);
  }
}
