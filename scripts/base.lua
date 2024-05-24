function takeYourTurn()
	local cars = GameState.cars
	local index = GameState.index
	local car = cars[index]
	local balance = car.balance
	GameState:buy_acceleration(2)
	GameState:buy_banana()

	local bananas = GameState.bananas
	-- print all the bananas localtions
	--print(ipairs(bananas))
	for i, banana in ipairs(bananas) do
		print(i, banana)
	end
end
