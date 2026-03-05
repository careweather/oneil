def iter_sum(a, n):
	sum = 0
	for i in range(int(n)):
		sum = sum + a
	return sum

import oneil
from oneil import MeasuredNumber

meters = oneil.units.meters
seconds = oneil.units.seconds

def calc_velocity(level):
	distance = MeasuredNumber(level * 10, meters)
	time = MeasuredNumber(level + 5, seconds)
	return distance / time