use effect::Effect;

#[derive(Debug, Clone)]
pub struct Item {
  pub name: String,
  pub cost: f64,
  pub effects: Vec<Effect>,
}

impl Item {
  pub fn new() -> Item {
    Item {
      name: "Unnamed".to_string(),
      cost: 0.0,
      effects: Vec::new(),
    }
  }
}
