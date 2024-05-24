function takeYourTurn()
	local cars = GameState.cars
	local index = GameState.index
	local car = cars[index]
	local balance = car.balance

	GameState:buy_acceleration(10)
end
