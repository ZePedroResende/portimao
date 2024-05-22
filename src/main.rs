#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
use std::sync::{Arc, Mutex};

use color_eyre::Result;
use mlua::{Error as LuaError, IntoLua, Value};
use mlua::{Function, Lua, UserData};
use serde::{Deserialize, Serialize};

const PLAYERS_REQUIRED: usize = 3;
const STARTING_BALANCE: u32 = 17500;
const FINISH_DISTANCE: u32 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Car {
    pub balance: u32,
    pub speed: u32,
    pub y: u32,
    pub shield: u32,
    #[serde(skip_serializing)]
    pub lua_script: String,
}

impl<'lua> IntoLua<'lua> for Car {
    fn into_lua(self, lua: &'lua Lua) -> Result<Value<'lua>, LuaError> {
        let table = lua.create_table()?;
        table.set("balance", self.balance)?;
        table.set("speed", self.speed)?;
        table.set("y", self.y)?;
        table.set("shield", self.shield)?;
        Ok(Value::Table(table))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Game {
    state: State,
    turns: usize,
    cars: Vec<Car>,
    bananas: Vec<u32>,
    logs: Vec<Log>,
    winner: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Action {
    Acceleration(u32),
    Banana(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum State {
    Waiting,
    Active,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Log {
    actions: Vec<Action>,
    balances: Vec<u32>,
    bananas: Vec<u32>,
    costs: Vec<u32>,
    current_car: usize,
    position: Vec<u32>,
    speeds: Vec<u32>,
}

impl Game {
    fn new() -> Self {
        Self {
            state: State::Waiting,
            turns: 1,
            cars: Vec::new(),
            bananas: Vec::new(),
            logs: Vec::new(),
            winner: None,
        }
    }

    fn register(&mut self, car: Car) {
        assert!(!(self.state != State::Waiting), "Game already started");

        self.cars.push(car);
        if self.cars.len() == PLAYERS_REQUIRED {
            self.state = State::Active;
        }
    }

    fn build_log(&self ) -> Log {
        Log {
            actions: vec![],
            balances: self.cars.iter().map(|c| c.balance).collect(),
            bananas: self.bananas.clone(),
            costs: vec![Self::get_accelerate_cost(1), Self::get_banana_cost(), Self::get_shell_cost()],
            current_car: self.get_index(),
            position: self.cars.iter().map(|c| c.y).collect(),
            speeds: self.cars.iter().map(|c| c.speed).collect(),
                    
                    
                    
        }
    }

    fn race(&mut self) {
        while self.state == State::Active {
            self.run_car();
            self.play_turn();
        }
        let log = serde_json::to_string(&self.logs).unwrap();
        let winner = serde_json::to_string(&self.winner).unwrap();
        println!("{}", serde_json::json!({"logs" : log, "winner" :winner}).to_string());
    }

    fn play_turn(&mut self) {
        assert!(!(self.state != State::Active), "Game not active");

        for (index, car) in &mut self.cars.iter_mut().enumerate() {
            // Update shields
            if car.shield > 0 {
                car.shield -= 1;
            }

            // Move car
            car.y += car.speed;

            // Check for banana collisions
            if let Some(pos) = self.bananas.iter().position(|&b| b == car.y) {
                car.speed = 0;
                self.bananas.remove(pos);
            }

            // Check for finish line
            if car.y >= FINISH_DISTANCE {
                self.state = State::Done;
                self.winner = Some(index);
                return;
            }
        }

        self.turns += 1;
    }

    fn run_car(&mut self) {
        self.logs.push(self.build_log());
        let lua = Lua::new();
        let index = self.turns % self.cars.len();
        let car: Car = self.cars[index].clone();

        let globals = lua.globals();

        let state = Arc::new(Mutex::new(self.to_owned()));
        let game_state = GameState(state.clone());

        lua.globals().set("GameState", game_state).unwrap();

        lua.load(&car.lua_script)
            .exec()
            .expect("Failed to load Lua script");

        let take_your_turn: Function = globals
            .get("takeYourTurn")
            .expect("Failed to get takeYourTurn function");

        let _: () = take_your_turn
            .call(())
            .expect("Failed to call takeYourTurn function");

        let new_state = state.lock().unwrap();
        self.cars.clone_from(&new_state.cars);
        self.bananas.clone_from(&new_state.bananas);
        self.logs.clone_from(&new_state.logs);
    }

    fn buy_acceleration(&mut self, car_index: usize, amount: u32) {
        let car = self.cars.get_mut(car_index).expect("Car failed");
        let cost = Self::get_accelerate_cost(amount);
        if car.balance >= cost {
            car.balance -= cost;
            car.speed += amount;

            self.logs.last_mut().unwrap().actions
                .push(Action::Acceleration(amount));
        }
    }

    fn buy_banana(&mut self, car_index: usize) {
        let car = &mut self.cars[car_index];
        let cost = Self::get_banana_cost();
        if car.balance >= cost {
            car.balance -= cost;
            self.bananas.push(car.y);

            self.logs.last_mut().unwrap().actions
                .push(Action::Banana(car_index));
        }
    }

    fn buy_shell(&mut self, car_index: usize) {
        let car = &mut self.cars[car_index];
        let cost = Self::get_shell_cost();
        if car.balance >= cost {
            car.balance -= cost;
        }
    }

    const fn get_banana_cost() -> u32 {
        // Simplified cost calculation
        200
    }

    const fn get_accelerate_cost(amount: u32) -> u32 {
        // Simplified cost calculation
        amount * 10
    }

    const fn get_shell_cost() -> u32 {
        // Simplified cost calculation
        200
    }

    fn get_index(&self) -> usize {
        self.turns % self.cars.len()
    }
}

struct GameState(Arc<Mutex<Game>>);

impl UserData for GameState {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("turns", |_, this| Ok(this.0.lock().unwrap().turns));
        fields.add_field_method_get("cars", |_, this| Ok(this.0.lock().unwrap().cars.clone()));
        fields.add_field_method_get("bananas", |_, this| {
            Ok(this.0.lock().unwrap().bananas.clone())
        });
        fields.add_field_method_get("index", |_, this| {
            let index = this.0.lock().unwrap().get_index() + 1;
            Ok(index)
        });
    }
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("buy_acceleration", |_, user_data, amount: u32| {
            {
                let mut lock = user_data.0.lock().unwrap();
                let index = lock.get_index();
                lock.buy_acceleration(index, amount);
            }
            Ok(())
        });

        methods.add_method_mut("buy_banana", |_, user_data, (): ()| {
            {
                let mut lock = user_data.0.lock().unwrap();
                let index = lock.get_index();
                lock.buy_banana(index);
            }
            Ok(())
        });
    }
}

fn main() {
    let mut game = Game::new();

    let lua_script_1 = std::fs::read_to_string("src/base.lua").unwrap();
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1.clone(),
    });
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1.clone(),
    });
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1,
    });

    game.race();
}
