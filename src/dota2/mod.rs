use item::Item;
use effect::{Effect, ExtraDamage, DamageDependency};
use hero::Hero;
mod from_dota2;

// manually create:
// MoM active (attack speed and damage amplification)
// dagon //cant be parsed because it uses level syntax
// mele / range bashes
// Manta
// fix manta ms seperately
// Vanguard damage block fix (json only shows chance not amount blocked)
// fix power tread
//

pub struct Dota2 {
  heroes: Vec<Hero>,
  items: Vec<Item>,
}

impl Dota2 {
  pub fn new() -> Dota2 {
    let heroes = from_dota2::parse_all_heroes("data/json/heroes/");
    let mut items = from_dota2::parse_all_items("data/json/items/");

    // Diffusal Blade is currently not parsed to json
    items.push(Item {
      name: "Diffusal Blade 1".to_string(),
      cost: 3150.0,
      effects: vec![Effect::Agility(20.0), Effect::Intelligence(6.0), Effect::ExtraDamage(ExtraDamage::Physical(25.0))],
    });
    items.push(Item {
      name: "Diffusal Blade 2".to_string(),
      cost: 3850.0,
      effects: vec![Effect::Agility(35.0), Effect::Intelligence(10.0), Effect::ExtraDamage(ExtraDamage::Physical(25.0))],
    });

    // Same for Dagon
    items.push(Item {
      name: "Dagon 1".to_string(),
      cost: 2720.0,
      effects: vec![Effect::Agility(3.0), Effect::Intelligence(13.0), Effect::Strength(3.0), Effect::AttackDamage(9.0)],
    });
    items.push(Item {
      name: "Dagon 5".to_string(),
      cost: 7720.0,
      effects: vec![Effect::Agility(3.0), Effect::Intelligence(25.0), Effect::Strength(3.0), Effect::AttackDamage(9.0)],
    });

    {
      // Fix the damage type for mkb because it defaults to physical
      let mut mkb = items.iter_mut().find(|i| &i.name[..] == "Monkey King Bar").unwrap();
      for effect in mkb.effects.iter_mut() {
        match *effect {
          Effect::ExtraDamage(ExtraDamage::Physical(amount)) => {
            *effect = Effect::ExtraDamage(ExtraDamage::Magical(amount));
            break;
          }
          _ => (),
        }
      }
    }
    {
      // Add Chain Lightning without bounces
      let mut maelstrom = items.iter_mut().find(|i| &i.name[..] == "Maelstrom").unwrap();
      maelstrom.effects.push(Effect::ExtraDamage(ExtraDamage::Magical(120.0 * 0.25)));
    }
    {
      // Add Chain Lightning without bounces
      let mut mjollnir = items.iter_mut().find(|i| &i.name[..] == "Mjollnir").unwrap();
      mjollnir.effects.push(Effect::ExtraDamage(ExtraDamage::Magical(150.0 * 0.25)));
    }
    {
      // Armlet active Unholy Strength
      let mut armlet = items.iter_mut().find(|i| &i.name[..] == "Armlet").unwrap();
      armlet.effects.push(Effect::AttackDamage(31.0));
      armlet.effects.push(Effect::Strength(25.0));
    }
    {
      // Add MoM active
      let mut mom = items.iter_mut().find(|i| &i.name[..] == "Mask of Madness").unwrap();
      mom.effects.push(Effect::AttackSpeed(100.0));
      mom.effects.push(Effect::AmplifyDamageTaken(0.3));
      mom.effects.push(Effect::MoveSpeedRelative(0.17));
    }
    {
      // Add damage block
      let mut vanguard = items.iter_mut().find(|i| &i.name[..] == "Vanguard").unwrap();
      vanguard.effects.push(Effect::DamageBlock(0.75, 40.0, 20.0));
    }
    {
      let mut crimson_guard = items.iter_mut().find(|i| &i.name[..] == "Crimson Guard").unwrap();
      crimson_guard.effects.push(Effect::DamageBlock(0.75, 40.0, 20.0));
      crimson_guard.effects.push(Effect::Armor(2.0));
    }

    Dota2 {
      heroes: heroes,
      items: items,
    }
  }
  pub fn get_heroes(&self) -> &Vec<Hero> {
    &self.heroes
  }
  pub fn get_items(&self) -> &Vec<Item> {
    &self.items
  }
  // TODO Implement this for the Iterator<Hero> trait to make it more generic
  pub fn get_hero_by_name(&self, name: &str) -> Option<&Hero> {
    self.heroes.iter().find(|hero| &hero.name[..] == name)
  }
  pub fn get_item_by_name(&self, name: &str) -> Option<&Item> {
    self.items.iter().find(|item| &item.name[..] == name)
  }
  pub fn get_maxed_out_heroes(&self) -> Vec<Hero> {
    let mut heroes: Vec<Hero> = self.heroes.clone();
    for hero in heroes.iter_mut() {
      hero.level = 25;
      hero.effects.add_effect(&Effect::Agility(20.0));
      hero.effects.add_effect(&Effect::Intelligence(20.0));
      hero.effects.add_effect(&Effect::Strength(20.0));
      if &hero.name[..] == "Alchemist" {
        hero.base_attack_time = 1.0;
      }
      if &hero.name[..] == "Lycan" {
        hero.base_attack_time = 1.5;
      }
      if &hero.name[..] == "Troll Warlord" {
        hero.base_attack_time = 1.55;
      }
      if &hero.name[..] == "Terror Blade" {
        hero.base_attack_time = 1.6;
        hero.starting_damage_min += 80.0;
        hero.starting_damage_max += 80.0;
      }
    }
    {
      // Explicit scope beacuse add_ability borrows heroes otherwise and we cant return it
      let mut add_ability =
        |name: &str, effect: Effect| heroes.iter_mut().find(|h| &h.name[..] == name).unwrap().effects.add_effect(&effect);
      // add_ability( "", Effect::() );
      add_ability("Juggernaut", Effect::CriticalStrike(0.35, 2.0));
      add_ability("Tiny", Effect::Armor(5.0));
      add_ability("Tiny", Effect::AttackDamage(150.0));
      add_ability("Tiny", Effect::AttackSpeed(-50.0));
      add_ability("Beastmaster", Effect::AttackSpeed(45.0));
      add_ability("Dragon Knight", Effect::Armor(12.0));
      add_ability("Dragon Knight", Effect::HPRegenerationAbsolute(5.0));
      add_ability("Sven",
                  Effect::DependencyAsAttackDamage(DamageDependency::BaseDamage, 2.0));
      add_ability("Sven", Effect::Armor(16.0));
      add_ability("Alchemist", Effect::HPRegenerationAbsolute(100.0));
      add_ability("Alchemist", Effect::ManaRegenerationAbsolute(12.0));
      add_ability("Brewmaster", Effect::Evasion(0.25));
      add_ability("Brewmaster", Effect::CriticalStrike(0.25, 2.0));
      add_ability("Night Stalker", Effect::MoveSpeedRelative(0.35));
      add_ability("Night Stalker", Effect::AttackSpeed(90.0));
      add_ability("Skeleton King or Wraith King",
                  Effect::CriticalStrike(0.15, 3.0));
      add_ability("Lycan", Effect::AttackSpeed(30.0));
      add_ability("Lycan",
                  Effect::DependencyAsAttackDamage(DamageDependency::BaseDamage, 0.3));
      add_ability("Lycan", Effect::CriticalStrike(0.3, 1.7));
      add_ability("Chaos Knight", Effect::CriticalStrike(0.1, 3.0));
      add_ability("Magnus",
                  Effect::DependencyAsAttackDamage(DamageDependency::BaseDamage, 0.5));
      add_ability("Abaddon", Effect::AttackSpeed(40.0));
      add_ability("Antimage",
                  Effect::ExtraDamage(ExtraDamage::Magical(64.0 * 0.6)));
      add_ability("Drow Ranger", Effect::Agility(80.0));
      add_ability("Drow Ranger",
                  Effect::DependencyAsAttackDamage(DamageDependency::Agility, 0.36));
      add_ability("Vengeful Spirit",
                  Effect::DependencyAsAttackDamage(DamageDependency::Agility, 0.36));
      add_ability("Riki",
                  Effect::DependencyAsExtraDamage(DamageDependency::Agility, ExtraDamage::Physical(1.25)));
      add_ability("Sniper",
                  Effect::ExtraDamage(ExtraDamage::Physical(90.0 * 0.4)));
      add_ability("Ursa",
                  Effect::DependencyAsAttackDamage(DamageDependency::HP, 0.07));
      add_ability("Troll Warlord", Effect::HP(100.0));
      add_ability("Troll Warlord", Effect::Armor(3.0));
      add_ability("Troll Warlord", Effect::AttackSpeed(4.0 * 34.0 + 180.0));
      add_ability("Troll Warlord",
                  Effect::ExtraDamage(ExtraDamage::Physical(50.0 * 0.1)));
      add_ability("Nevermore", Effect::AttackDamage(36.0 * 2.0));
      add_ability("Faceless Void", Effect::Evasion(0.25));
      add_ability("Faceless Void",
                  Effect::ExtraDamage(ExtraDamage::Physical(70.0 * 0.25)));
      add_ability("Phantom Assassin", Effect::Evasion(0.5));
      add_ability("Phantom Assassin", Effect::CriticalStrike(0.15, 4.5));
      add_ability("Clinkz", Effect::AttackSpeed(130.0));
      add_ability("Clinkz", Effect::ExtraDamage(ExtraDamage::Physical(60.0)));
      add_ability("Spectre", Effect::AmplifyDamageTaken(-0.22));
      add_ability("Windrunner", Effect::AttackSpeed(400.0));
      add_ability("Lina", Effect::AttackSpeed(85.0 * 3.0));
      add_ability("Tidehunter", Effect::DamageBlock(1.0, 48.0, 48.0));
    }
    heroes
  }
}
