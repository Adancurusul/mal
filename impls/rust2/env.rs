use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::MalType;

// Environment structure with support for nested scopes
#[derive(Debug, Clone)]
pub struct Env {
    data: HashMap<String, MalType>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    // Create a new environment with optional outer scope
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Env {
        Env {
            data: HashMap::new(),
            outer,
        }
    }

    // Create a new environment with bindings
    pub fn with_bindings(outer: Option<Rc<RefCell<Env>>>, binds: &[String], exprs: &[MalType]) -> Result<Env, String> {
        let mut env = Env::new(outer);
        
        for (bind, expr) in binds.iter().zip(exprs.iter()) {
            env.set(bind, expr.clone());
        }
        
        Ok(env)
    }

    // Get a value from the environment
    pub fn get(&self, key: &str) -> Option<MalType> {
        match self.data.get(key) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(outer) = &self.outer {
                    outer.borrow().get(key)
                } else {
                    None
                }
            }
        }
    }

    // Set a value in the current environment
    pub fn set(&mut self, key: &str, val: MalType) -> MalType {
        self.data.insert(key.to_string(), val.clone());
        val
    }

    // Find the environment that contains the key
    pub fn find(&self, key: &str) -> Option<Rc<RefCell<Env>>> {
        if self.data.contains_key(key) {
            Some(Rc::new(RefCell::new(self.clone())))
        } else if let Some(outer) = &self.outer {
            outer.borrow().find(key)
        } else {
            None
        }
    }
} 