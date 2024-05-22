function takeYourTurn()
	local cars = GameState.cars
	print(cars)
	local index = GameState.index
	print(index)
	local car = cars[index]
	print("Car balance: " .. car.balance)
	print("Car speed: " .. car.speed)
	print("Car y: " .. car.y)
	print("Car shield: " .. car.shield)
	local balance = car.balance
	print("Balance: " .. balance)
	GameState:buy_acceleration(50)
	GameState:buy_banana()
end
