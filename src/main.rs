#![allow(dead_code)]

extern crate rustc_serialize;

// use std::fmt;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::iter::Iterator;
use hero::{Hero, Attribute};
use item::Item;
use effect::Effect;
mod item;
mod effect;
mod hero;
mod combinatorics;
mod dota2;


// TODO: Where to put illusions... Can we add illu dps*ehp to  the heros?
// When will mom active increase dps * ehp


fn items_to_string(items: &Vec<&Item>) -> String {
  let mut string = String::new();
  let mut first = true;
  for item in items.iter() {
    if !first {
      string.push_str(", ");
    }
    string.push_str(&item.name[..]);
    first = false;
  }
  string
}

// Does item influence the evaluation of a hero.
// For example, does Ring of Basillius influence the EHP of a hero?
// Unconditionally because currently in dota most items like Armor or Intelligence have only unconditional influences
// that always exist no matter the state of the hero.
// An exception would be for example damage block or move speed which not always grant a bonus to a hero.
// fn influences_unconditionally<T: PartialEq>( hero: Hero, item: &Item, evaluate: |hero: &Hero| -> T ) -> bool {
// TODO: add macro to make writing things with it shorter
fn influences_unconditionally<T: PartialEq, F: Fn(&Hero) -> T>(item: &Item, hero: &Hero, evaluate: F) -> bool {
  let mut hero = hero.clone();
  let result_without_item = evaluate(&hero);
  hero.add_item(item);
  let result_with_item = evaluate(&hero);
  return result_without_item.ne(&result_with_item);
}

// Runs evaluate on hero with the items from Combinations and sorts the result.
// Can only be used on "unconditional" items.
// For example to get the best dps with 2 items from all items costing less than 3000:
// for &(dps,ref items) in simple_best_and_filter( &hero, dota2.get_items().iter().filter(
// |item| item.cost < 3000.0), 2, |h| h.damage_per_second() ).iter().take(10) { println!("{} {}\n", dps, items) }
fn simple_best_and_filter<'a,
                          ItemIterator: Iterator<Item = &'a Item>,
                          ValueIterator: Iterator<Item = usize>,
                          Result: PartialOrd + PartialEq,
                          Evaluation: Fn(&Hero) -> Result>
  (hero: &Hero,
   items: ItemIterator,
   values: ValueIterator,
   evaluate: Evaluation)
   -> Vec<(Result, Vec<&'a Item>)> {
  let mut result = Vec::new();
  // Filter all items that dont influence the evaluation.
  let items: Vec<&'a Item> = items.filter(|item: &&Item| influences_unconditionally(*item, hero, |hero: &Hero| evaluate(hero)))
    .collect();
  for i in values.flat_map(|size| combinatorics::CombinationsWithReplacement::new(items.clone(), size)) {
    let mut hero = hero.clone();
    hero.add_items(&i);
    result.push((evaluate(&hero), i));
  }
  result.sort_by(|&(ref x1, _), &(ref x2, _)| {
    match x2.partial_cmp(x1) {
      None => panic!(),
      Some(ordering) => ordering,
    }
  });
  result
}

// Does not automatically filter anything and the evaluation function can take items into account.
// Example: Get item combination that gives the most ehp * dps for combinations that you can buy as starting items
//
// for &(dps,ref items) in advanced_best( &hero,
// dota2.get_items().iter().filter(
// |&item| item. cost < 650.0 && influences_unconditionally(
// Hero::new(), item, |h| h.damage_per_second() * h.effective_hp_physical() ) ),	6,
// |h, _| h.damage_per_second() * h.effective_hp_physical(),
// |i| i.iter().fold( 0.0, |acc, item| acc + item.cost ) < 650.0 )
// .iter().take(10) {
// println!("{} {}\n", dps, items)
// }
//

fn advanced_best<'a,
                 ItemIterator: Iterator<Item = &'a Item>,
                 ValueIterator: Iterator<Item = usize>,
                 Result: PartialOrd + PartialEq,
                 Evaluation: Fn(&Hero, &Vec<&Item>) -> Result,
                 Filter: Fn(&Vec<&Item>) -> bool>
  (hero: &Hero,
   items: ItemIterator,
   values: ValueIterator,
   evaluate: Evaluation,
   filter: Filter)
   -> Vec<(Result, Vec<&'a Item>)> {
  let mut result = Vec::new();
  let items: Vec<&Item> = items.collect();
  for i in values.flat_map(|size| combinatorics::CombinationsWithReplacement::new(items.clone(), size))
    .filter(filter) {
    let mut hero = hero.clone();
    hero.add_items(&i);
    result.push((evaluate(&hero, &i), i));
  }
  result.sort_by(|&(ref x1, _), &(ref x2, _)| {
    match x2.partial_cmp(x1) {
      None => panic!(),
      Some(ordering) => ordering,
    }
  });
  result
}

fn advanced_best_optimized<'a,
                           ItemIterator: Iterator<Item = &'a Item>,
                           ValueIterator: Iterator<Item = usize>,
                           Result: PartialOrd + PartialEq,
                           Evaluation: Fn(&Hero, &Vec<&Item>) -> Result,
                           Filter: Fn(&Vec<&Item>) -> bool>
  (hero: &Hero,
   items: ItemIterator,
   values: ValueIterator,
   evaluate: Evaluation,
   filter: Filter,
   number_of_results: usize)
   -> Vec<(Result, Vec<&'a Item>)> {
  let mut results = Vec::<(Result, Vec<&'a Item>)>::with_capacity(number_of_results); //A sorted list of the n best results
  let items: Vec<&Item> = items.collect();
  for i in values.flat_map(|size| combinatorics::CombinationsWithReplacement::new(items.clone(), size))
    .filter(filter) {
    let mut hero = hero.clone();
    hero.add_items(&i);
    let result = evaluate(&hero, &i);
    if results.len() >= number_of_results {
      let length = results.len();
      let mut position = length;
      while position >= 1 {
        match results[position - 1] {
          (ref existing_result, _) if *existing_result < result => position -= 1,
          _ => break,
        }
      }
      if position < length {
        results.pop();
        results.insert(position, ((result, i)));
      }
    } else {
      results.push((result, i));
    }
  }
  results
}

// Computes by how much an item combination increases evaluate per gold it costs.
fn best_items_per_gold<'a,
                       ItemIterator: Iterator<Item = &'a Item>,
                       ValueIterator: Iterator<Item = usize>,
                       Evaluation: Fn(&Hero, &Vec<&Item>) -> f64>
  (hero: &Hero,
   items: ItemIterator,
   values: ValueIterator,
   evaluate: &Evaluation)
   -> Vec<(f64, Vec<&'a Item>)> {
  let before = evaluate(hero, &Vec::new());
  advanced_best(hero,
                items,
                values,
                |hero, items| (evaluate(hero, items) - before) / items.iter().fold(0.0, |acc, item| acc + item.cost),
                |_| true)
}

fn best_items_per_gold_output<'a,
                              ItemIterator: Iterator<Item = &'a Item>,
                              ValueIterator: Iterator<Item = usize>,
                              Evaluation: Fn(&Hero, &Vec<&Item>) -> f64>
  (hero: &Hero,
   items: ItemIterator,
   values: ValueIterator,
   evaluate: &Evaluation,
   output_count: usize) {
  let result = best_items_per_gold(hero, items, values, evaluate);
  for &(efficiency, ref items) in result.iter().take(output_count) {
    let mut h = hero.clone();
    h.add_items(items);
    println!("{:?} {} {:?}\n",
             evaluate(&h, items),
             items_to_string(items),
             efficiency)
  }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn main() {
  let dota2 = dota2::Dota2::new();
  let heroes = dota2.get_heroes();
  let maxed_out_heroes = dota2.get_maxed_out_heroes();
  let mut items = dota2.get_items().clone();
  let fish_bones = Item {
    name: "Fish Bones".to_string(),
    cost: 100.0,
    effects: vec![Effect::AttackDamage(3.0), Effect::Armor(1.0)],
  };

  // for i in items.iter_mut() {
  // for j in range(0, i.effects.len()).rev() {
  // match i.effects[j] { Effect::Evasion(_) => {i.effects.remove(j);}, _ => () }
  // }
  // }

  let filter_items = |i: &Item| -> bool {
    let name = i.name.as_str();
    name != "Rapier" && name != "Mask of Madness" && name != "Armlet"
  };

  for hero in heroes {
    let mut hero1 = hero.clone();
    hero1.level = 1;
    let dps0 = hero1.damage_per_second_physical();
    let ehp0 = hero1.effective_hp_physical();
    let result0 = dps0 * ehp0;
    hero1.add_item(&fish_bones);
    let dps1 = hero1.damage_per_second_physical();
    let ehp1 = hero1.effective_hp_physical();
    let result1 = dps1 * ehp1;

    let mut hero2 = hero.clone();
    hero2.level = 1;
    hero2.add_item(dota2.get_item_by_name("Branches").unwrap());
    hero2.add_item(dota2.get_item_by_name("Branches").unwrap());
    let dps2 = hero2.damage_per_second_physical();
    let ehp2 = hero2.effective_hp_physical();
    let result2 = dps2 * ehp2;

    println!("{}\nDps: Default: {:.3}, Fish Bones: {:.3}, Branches: {:.3}\nEhp: Default: {:.0}, Fish Bones: {:.0}, Branches: {:.0}\n",
             hero.name,
             dps0,
             dps1,
             dps2,
             ehp0,
             ehp1,
             ehp2);
  }

  // result.sort_by( |&(_,f1), &(_,f2)| match f2.partial_cmp(&f1) { None => panic!(), Some(ordering) => ordering } );
  // for a in result.iter() {
  // println!("{:?}", a);
  // }

}
