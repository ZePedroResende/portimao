use mlua::prelude::LuaError;
use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

const STARTING_BALANCE: u128 = 17500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Car {
    pub balance: u128,
    pub speed: u32,
    pub y: u32,
    #[serde(skip_serializing)]
    pub lua_script: String,
    pub name: String,
}

impl Car {
    pub const fn new(script: String, name: String) -> Self {
        Self {
            balance: STARTING_BALANCE,
            speed: 0,
            y: 0,
            lua_script: script,
            name,
        }
    }
}

impl<'lua> IntoLua<'lua> for Car {
    fn into_lua(self, lua: &'lua Lua) -> color_eyre::Result<Value<'lua>, LuaError> {
        let table = lua.create_table()?;
        table.set("balance", self.balance)?;
        table.set("speed", self.speed)?;
        table.set("y", self.y)?;
        Ok(Value::Table(table))
    }
}
