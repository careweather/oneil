def iter_sum(a, n):
	sum = 0
	for i in range(int(n)):
		sum = sum + a
	return sum

from oneil import Unit, MeasuredNumber

meters = Unit(dimensions={'m': 1}, display_unit='m')

seconds = Unit(dimensions={'s': 1}, display_unit='s')

def calc_velocity(level):
	distance = MeasuredNumber(level * 10, meters)
	time = MeasuredNumber(level + 5, seconds)
	return distance / time