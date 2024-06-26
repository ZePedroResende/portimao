use mlua::prelude::LuaError;
use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::car::Car;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub actions: Vec<Action>,
    pub bananas: Vec<u32>,
    pub costs: Vec<u128>,
    pub current_car: usize,
    pub cars: Vec<Car>,
    pub actions_sold: Vec<u128>,
}

impl Log {
    pub fn add_info(
        &mut self,
        bananas: Vec<u32>,
        costs: Vec<u128>,
        current_car: usize,
        cars: Vec<Car>,
        actions_sold: Vec<u128>,
    ) {
        self.bananas = bananas;
        self.costs = costs;
        self.current_car = current_car;
        self.cars = cars;
        self.actions_sold = actions_sold;
    }
}

impl<'lua> IntoLua<'lua> for Log {
    fn into_lua(self, lua: &'lua Lua) -> color_eyre::Result<Value<'lua>, LuaError> {
        let table = lua.create_table()?;
        table.set("actions", self.actions)?;
        table.set("bananas", self.bananas)?;
        table.set("costs", self.costs)?;
        table.set("car", self.cars)?;
        table.set("actions_sold", self.actions_sold)?;
        Ok(Value::Table(table))
    }
}
