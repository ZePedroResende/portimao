use mlua::{Error as LuaError, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(usize)]
pub enum Action {
    Acceleration(u32) = 0,
    Banana(usize) = 1,
    Shell(usize) = 2,
}

impl<'lua> IntoLua<'lua> for Action {
    fn into_lua(self, lua: &'lua Lua) -> color_eyre::Result<Value<'lua>, LuaError> {
        let table = lua.create_table()?;
        match self {
            Action::Acceleration(amount) => {
                table.set("type", "acceleration")?;
                table.set("amount", amount)?;
            }
            Action::Banana(index) => {
                table.set("type", "banana")?;
                table.set("index", index)?;
            }
            Action::Shell(index) => {
                table.set("type", "shell")?;
                table.set("index", index)?;
            }
        }

        Ok(Value::Table(table))
    }
}

impl Action {
    pub fn into_usize(self) -> usize {
        match self {
            Action::Acceleration(_) => 0,
            Action::Banana(_) => 1,
            Action::Shell(_) => 2,
        }
    }
}
