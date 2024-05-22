use mlua::{Function, Lua, Table};

const PLAYERS_REQUIRED: usize = 3;
const POST_SHELL_SPEED: u32 = 1;
const STARTING_BALANCE: u32 = 17500;
const FINISH_DISTANCE: u32 = 1000;
const BANANA_SPEED_MODIFIER: f64 = 0.5;

#[derive(Debug, Clone)]
struct Car {
    pub balance: u32,
    pub speed: u32,
    pub y: u32,
    pub shield: u32,
    pub lua_script: String,
}

#[derive(Debug, Clone)]
struct Game {
    state: State,
    turns: usize,
    cars: Vec<Car>,
    bananas: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Waiting,
    Active,
    Done,
}

impl Game {
    fn new() -> Self {
        Self {
            state: State::Waiting,
            turns: 1,
            cars: Vec::new(),
            bananas: Vec::new(),
        }
    }

    fn register(&mut self, car: Car) {
        if self.state != State::Waiting {
            panic!("Game already started");
        }

        self.cars.push(car);
        if self.cars.len() == PLAYERS_REQUIRED {
            self.state = State::Active;
        }
    }

    fn play_turn(&mut self) {
        if self.state != State::Active {
            panic!("Game not active");
        }

        self.run_car(self.turns % self.cars.len());

        for car in &mut self.cars {
            // Update shields
            if car.shield > 0 {
                car.shield -= 1;
            }

            // Move car
            car.y += car.speed;

            // Check for banana collisions
            if let Some(pos) = self.bananas.iter().position(|&b| b == car.y) {
                car.speed = (car.speed as f64 * BANANA_SPEED_MODIFIER) as u32;
                self.bananas.remove(pos);
            }

            // Check for finish line
            if car.y >= FINISH_DISTANCE {
                self.state = State::Done;
                println!("Car finished: {:?}", car);
                return;
            }
        }

        self.turns += 1;
    }

    fn run_car(&mut self, index: usize) {
        let car: Car = self.cars[index].clone();
        let lua = Lua::new();

        let globals = lua.globals();
        let game_table = lua.create_table().unwrap();

        let car_data_table = lua.create_table().unwrap();
        car_data_table.set("balance", car.balance).unwrap();
        car_data_table.set("speed", car.speed).unwrap();
        car_data_table.set("y", car.y).unwrap();
        car_data_table.set("shield", car.shield).unwrap();
        car_data_table.set("index", index).unwrap();

        lua.create_function_mut(|_, (car_index, amount): (usize, u32)| {
            self.buy_acceleration(car_index, amount);
            Ok(())
        });

        //let action_table = lua.create_table().unwrap();
        //action_table.set("speed", false).unwrap();

        game_table.set("CarData", car_data_table).unwrap();

        globals.set("Game", game_table).unwrap();

        lua.load(&car.lua_script)
            .exec()
            .expect("Failed to load Lua script");

        let take_your_turn: Function = globals
            .get("takeYourTurn")
            .expect("Failed to get takeYourTurn function");

        let _: () = take_your_turn
            .call(())
            .expect("Failed to call takeYourTurn function");
    }

    fn buy_acceleration(&mut self, car_index: usize, amount: u32) {
        let car = self.cars.get_mut(car_index).expect("Car failed");
        let cost = Self::get_accelerate_cost(amount);
        if car.balance >= cost {
            car.balance -= cost;
            car.speed += amount;
        }
    }

    fn buy_banana(&mut self, car_index: usize) {
        let car = &mut self.cars[car_index];
        let cost = Self::get_banana_cost();
        if car.balance >= cost {
            car.balance -= cost;
            self.bananas.push(car.y);
        }
    }

    fn get_accelerate_cost(amount: u32) -> u32 {
        // Simplified cost calculation
        amount * 10
    }

    fn get_banana_cost() -> u32 {
        // Simplified cost calculation
        200
    }
}

fn main() {
    let mut game = Game::new();

    let lua_script_1 = r#"
    function takeYourTurn()
        local car = Game.CarData
        car.y = car.y + car.speed
        car.balance = car.balance - 10
    end
    "#;

    let lua_script_2 = r#"
    function takeYourTurn()
        local car = Monaco.CarData
        car.y = car.y + car.speed + 1
        car.balance = car.balance - 15
    end
    "#;
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1.to_string(),
    });
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1.to_string(),
    });
    game.register(Car {
        balance: STARTING_BALANCE,
        speed: 0,
        y: 0,
        shield: 0,
        lua_script: lua_script_1.to_string(),
    });

    while game.state == State::Active {
        game.play_turn();
    }
}
