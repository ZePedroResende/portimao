function takeYourTurn()
	local cars = GameState.cars
	local index = GameState.index
	local car = cars[index]
	local balance = car.balance
	GameState:buy_acceleration(1)
	GameState:buy_banana()

	if index == 3 then
		GameState:buy_shell(2)
	end
end
