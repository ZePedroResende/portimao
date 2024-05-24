use std::io::Write;
use std::sync::{Arc, Mutex};

use mlua::{Function, Lua, UserData};
use serde::{Deserialize, Serialize};

use crate::car::Car;
use crate::log::Log;
use std::fs::File;

const PLAYERS_REQUIRED: usize = 3;
const FINISH_DISTANCE: u32 = 1000;

const ACCELERATE_TARGET_PRICE: u128 = 10;
const ACCELERATE_PER_TURN_DECREASE: f64 = 0.33;
const ACCELERATE_SELL_PER_TURN: u128 = 2;

const BANANA_TARGET_PRICE: u128 = 200;
const BANANA_PER_TURN_DECREASE: f64 = 0.33;
const BANANA_SELL_PER_TURN: f64 = 0.2;

const SHELL_TARGET_PRICE: u128 = 200;
const SHELL_PER_TURN_DECREASE: f64 = 0.33;
const SHELL_SELL_PER_TURN: f64 = 0.2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    state: State,
    turns: usize,
    cars: Vec<Car>,
    bananas: Vec<u32>,
    logs: Vec<Log>,
    winner: Option<usize>,
    actions_sold: Vec<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(usize)]
pub enum Action {
    Acceleration(u32) = 0,
    Banana(usize) = 1,
    Shell(usize) = 2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum State {
    Waiting,
    Active,
    Done,
}

impl Game {
    pub(crate) fn new() -> Self {
        Self {
            state: State::Waiting,
            turns: 1,
            cars: Vec::new(),
            bananas: Vec::new(),
            logs: Vec::new(),
            winner: None,
            actions_sold: vec![0; 3],
        }
    }

    pub(crate) fn register(&mut self, car: Car) {
        assert!(!(self.state != State::Waiting), "Game already started");

        self.cars.push(car);
        if self.cars.len() == PLAYERS_REQUIRED {
            self.state = State::Active;
        }
    }

    pub(crate) fn race(&mut self) {
        if self.state != State::Active {
            panic!("Not enough players to start the game");
        }
        self.log_turn();
        while self.state == State::Active {
            self.run_car();
            self.play_turn();
        }

        self.export_log();
    }

    fn play_turn(&mut self) {
        assert!(!(self.state != State::Active), "Game not active");

        for (index, car) in &mut self.cars.iter_mut().enumerate() {
            // Update shields

            // Move car
            let car_old_position = car.y;
            let car_new_position = car.y + car.speed;

            car.y += car.speed;

            // Check for banana collisions
            if let Some(pos) = self
                .bananas
                .iter()
                .position(|&b| car_old_position < b && car_new_position >= b)
            {
                car.speed = 0;
                car.y = self.bananas[pos];
                self.bananas.remove(pos);
            }

            // Check for finish line
            if car.y >= FINISH_DISTANCE {
                self.state = State::Done;
                self.winner = Some(index);
                break;
            }
        }
        self.log_turn();

        self.turns += 1;
    }

    fn log_turn(&mut self) {
        let index = self.get_index();
        if self.logs.is_empty() {
            self.logs.push(Log::default());
        }

        let prices = vec![
            self.get_accelerate_cost(1),
            self.get_banana_cost(),
            self.get_shell_cost(1),
        ];

        self.logs.last_mut().unwrap().add_info(
            self.bananas.clone(),
            prices,
            index,
            self.cars.clone(),
            self.actions_sold.clone(),
        );
    }

    fn run_car(&mut self) {
        self.logs.push(Log::default());
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

        let take_your_turn: Result<Function, mlua::prelude::LuaError> = globals.get("takeYourTurn");

        if let Err(e) = take_your_turn {
            println!("Erroron getting take_your_turn function: {:?}", e);
            return;
        }

        let result: Result<(), mlua::prelude::LuaError> = take_your_turn.unwrap().call(());

        if let Ok(()) = result {
            let new_state = state.lock().unwrap();
            self.cars.clone_from(&new_state.cars);
            self.bananas.clone_from(&new_state.bananas);
            self.logs.clone_from(&new_state.logs);
            self.actions_sold.clone_from(&new_state.actions_sold);
        } else {
            println!("Error on calling take_your_turn function: {:?}", result);
        }
    }

    fn buy_acceleration(&mut self, car_index: usize, amount: u32) {
        let cost = self.get_accelerate_cost(amount);
        let car = self.cars.get_mut(car_index).expect("Car failed");
        if car.balance >= cost {
            car.balance -= cost;
            car.speed += amount;

            self.actions_sold[0] += amount as u128;
            self.logs
                .last_mut()
                .unwrap()
                .actions
                .push(Action::Acceleration(amount));
        }
    }

    fn buy_banana(&mut self, car_index: usize) {
        let cost = self.get_banana_cost();
        let car = &mut self.cars[car_index];
        if car.balance >= cost && !self.bananas.contains(&car.y) {
            car.balance -= cost;

            self.actions_sold[1] += 1;
            self.bananas.push(car.y);
            self.bananas.sort();

            self.logs
                .last_mut()
                .unwrap()
                .actions
                .push(Action::Banana(car_index));
        }
    }

    fn buy_shell(&mut self, car_index: usize, amount: u32) {
        let cost = self.get_shell_cost(amount);
        let cars = self.cars.clone();
        {
            let car = &mut self.cars.get_mut(car_index).unwrap();

            if car.balance < cost {
                return;
            }

            car.balance -= cost;
            self.actions_sold[2] += amount as u128;
        }

        // lets enum the cars with their index and remove the current car
        let mut remaining_cars = cars.into_iter().enumerate().collect::<Vec<_>>();
        remaining_cars.remove(car_index);

        // lets filter the cars that are in front of the current car
        let mut cars_in_front: Vec<_> = remaining_cars
            .iter()
            .filter(|(_, adversary_car)| self.cars[car_index].y <= adversary_car.y)
            .collect();

        // lets sort the cars by their y position
        cars_in_front.sort_by(|(_, a), (_, b)| a.y.cmp(&b.y));

        for _ in 0..amount {
            if !self.bananas.is_empty() {
                let b = self.bananas.clone();

                let bananas: Vec<_> = b
                    .iter()
                    .enumerate()
                    .filter(|&(_, b)| *b > self.cars[car_index].y && *b <= cars_in_front[0].1.y)
                    .collect();

                if !bananas.is_empty() {
                    let pos = bananas.first().unwrap().0;
                    self.bananas.remove(pos);
                    continue;
                }
            }

            //// lets hit the first car in front of the current car with a shell removing its speed
            if let Some((index, _)) = cars_in_front.first() {
                self.cars[*index].speed = 0;
            }
        }
    }

    fn get_accelerate_cost(&self, amount: u32) -> u128 {
        let actions_sold = self.actions_sold[0];
        let mut sum = 0;
        for i in 0..amount {
            sum += Self::compute_action_price(
                ACCELERATE_TARGET_PRICE as f64,
                ACCELERATE_PER_TURN_DECREASE,
                self.turns as u64,
                actions_sold + i as u128,
                ACCELERATE_SELL_PER_TURN as f64,
            ) as u128;
        }

        sum
    }

    fn get_banana_cost(&self) -> u128 {
        let actions_sold = self.actions_sold[1];
        Self::compute_action_price(
            BANANA_TARGET_PRICE as f64,
            BANANA_PER_TURN_DECREASE,
            self.turns as u64,
            actions_sold,
            BANANA_SELL_PER_TURN,
        ) as u128
    }

    fn get_shell_cost(&self, amount: u32) -> u128 {
        let actions_sold = self.actions_sold[2];
        let mut sum = 0;
        for i in 0..amount {
            sum += Self::compute_action_price(
                SHELL_TARGET_PRICE as f64,
                SHELL_PER_TURN_DECREASE,
                self.turns as u64,
                actions_sold + i as u128,
                SHELL_SELL_PER_TURN,
            ) as u128;
        }

        sum
    }

    fn compute_action_price(
        target_price: f64,
        per_turn_price_decrease: f64,
        turns_since_start: u64,
        sold: u128,
        sell_per_turn: f64,
    ) -> f64 {
        // Compute the intermediate value
        let intermediate_value =
            (turns_since_start - 1) as f64 - ((sold + 1) as f64 / sell_per_turn);

        // Compute the price multiplier using exponential and logarithmic functions

        let pre_ln = 1.0 - per_turn_price_decrease;
        let ln = pre_ln.ln();
        let price_multiplier = (ln * intermediate_value).exp();

        // Compute the action price

        target_price * price_multiplier
    }

    fn get_index(&self) -> usize {
        self.turns % self.cars.len()
    }

    pub fn export_log(&self) {
        let json = serde_json::json!({"logs" : self.logs, "winner" :self.winner});

        println!("{}", json);

        let time_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let filename = format!("logs/logs_{}.json", time_now);

        File::create(filename)
            .unwrap()
            .write_all(json.to_string().as_bytes())
            .unwrap();
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

        methods.add_method_mut("buy_shell", |_, user_data, amount: u32| {
            {
                let mut lock = user_data.0.lock().unwrap();
                let index = lock.get_index();
                lock.buy_shell(index, amount);
            }
            Ok(())
        });

        methods.add_method("get_accelerate_cost", |_, user_data, amount: u32| {
            let lock = user_data.0.lock().unwrap();
            let cost = lock.get_accelerate_cost(amount);
            Ok(cost)
        });

        methods.add_method("get_banana_cost", |_, user_data, (): ()| {
            let lock = user_data.0.lock().unwrap();
            let cost = lock.get_banana_cost();
            Ok(cost)
        });
        methods.add_method("get_shell_cost", |_, user_data, amount: u32| {
            let lock = user_data.0.lock().unwrap();
            let cost = lock.get_shell_cost(amount);
            Ok(cost)
        });
    }
}
