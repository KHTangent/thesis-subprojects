using Plots


function main()
	if length(ARGS) < 1
		println("Please spesify a filename")
		return
	end	
	transmit_times, latencies = read_file(ARGS[1])
	total_duration = round(Int, transmit_times[end] - transmit_times[1])
	one_second_length = div(length(transmit_times), total_duration)
	average_latency = sum(latencies)/length(latencies)
	variance = sum(map(e -> (e - average_latency)^2, latencies)) / length(latencies)
	standard_deviation = sqrt(variance)

	println("  Summary  ")
	println("===========")
	println("Test duration (s):        $(total_duration)")
	println("Number of packets:        $(length(latencies))")
	println("Average latency (s):      $(average_latency)")
	println("Average latency (µs):     $(average_latency * 10^6)")
	println("Standard deviation (s):   $(standard_deviation)")
	println("Standard deviation (µs):  $(standard_deviation * 10^6)")

	offset_value = transmit_times[1]
	map!(e -> e - offset_value, transmit_times, transmit_times)
	map!(e -> e * 10^6, latencies, latencies)

	gaston()
	plot(scatter(transmit_times, latencies))
	if endswith(ARGS[end], ".png")
		savefig(ARGS[end])
	else
		gui()
		readline()
	end
end

function read_file(filename::AbstractString)
	transmit_times = []
	latencies = []
	open(filename, "r") do io
		while true
			try
				transmit_time = read(io, Float64)
				arrival_time = read(io, Float64)
				append!(transmit_times, transmit_time)
				append!(latencies, arrival_time - transmit_time)
			catch
				break
			end
		end
	end
	return transmit_times, latencies
end

main()
