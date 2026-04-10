use crate::engine::engine::Engine;
use std::collections::HashMap;

#[derive(Clone)]
pub struct UserAccount {
    pub engine: Engine,
}

pub struct MultiUserEngine {
    pub users: HashMap<String, UserAccount>,
}

impl MultiUserEngine {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn get_or_create_user(&mut self, user_id: &str, initial_balance: f64) -> &mut UserAccount {
        self.users
            .entry(user_id.to_string())
            .or_insert_with(|| UserAccount {
                engine: Engine::new(initial_balance),
            })
    }

    pub fn get_user(&self, user_id: &str) -> Option<&UserAccount> {
        self.users.get(user_id)
    }

    pub fn get_user_mut(&mut self, user_id: &str) -> Option<&mut UserAccount> {
        self.users.get_mut(user_id)
    }

    pub fn get_all_users(&self) -> Vec<String> {
        self.users.keys().cloned().collect()
    }
}

impl Default for MultiUserEngine {
    fn default() -> Self {
        Self::new()
    }
}
