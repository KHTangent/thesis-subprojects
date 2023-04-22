#!/usr/bin/env python3
"""
This script is much more complicated than needed because I was too dumb to export JSON
data from the data postprocessor
This script is undocumented and not very readable, and should probably not be used
"""

import os
import re

input_path = "~/Development/thesis-subprojects/data/Testplots2/"


def parse_test(testname: str):
    testfile = os.path.join(input_path, testname, "summary.txt")
    parsed = []
    with open(testfile) as f:
        results = f.read().strip().split("\n\n")
        for result in results:
            parsed.append(parse_result(testname, result))
    return parsed


title_regex = re.compile(r"d(\d+)-l(\d+)-(\d+)")


def parse_result(title: str, result: str):
    lines = result.split("\n")
    parsed_title = title_regex.search(lines[0])
    parsed = {
        "title": title,
        "title_duration": parsed_title.group(1),
        "pps": int(parsed_title.group(2)),
        "test_no": int(parsed_title.group(3)),
        # Subtract 2 here because of bug in processor
        "duration": float(lines[2].split(" ")[2]) - 2,
        "packets": int(lines[3].split(" ")[2]),
        "latency": parse_triplet(lines[4].split(" ")[2]),
        "standard_deviation": float(lines[5].split(" ")[2]),
        "anomaly_treshold_t": float(lines[6].split(" ")[2]),
        "anomaly_treshold_n": int(lines[6].split(" ")[-1]),
        "anomaly_count": int(lines[7].split(" ")[2]),
    }
    parsed["packet_loss"] = 1 - \
        (parsed["packets"] / (parsed["pps"] * parsed["duration"]))
    if parsed["anomaly_count"] > 0:
        parsed["anomaly_duration_avg"] = float(lines[8].split(" ")[3])
        parsed["anomaly_latency_avg"] = parse_triplet(lines[9].split(" ")[4])
        parsed["anomaly_latency_max"] = parse_triplet(lines[10].split(" ")[4])
    return parsed


def parse_triplet(triplet: str):
    parsed = triplet.split("/")
    return (float(parsed[0]), float(parsed[1]), float(parsed[2]))


def tests_to_latex_table(tests: dict, pps: int, test_count: int, val: callable):
    output = "\\begin{tiny}\n"
    output += "\\begin{tabularx}{\\linewidth}{ |X"
    output += ("|l" * (test_count + 1)) + "| }\n"
    output += "\\hline\n"
    output += " & " + " & ".join([str(i)
                                 for i in range(test_count)]) + "\\\\\n"
    for testname in tests:
        output += "\\hline\n" + testname
        for test in tests[testname]:
            if test["pps"] != pps:
                continue
            output += " & $" + str(val(test)) + "$"
        output += "\\\\\n"
    output += "\\hline\n"
    output += "\\end{tabularx}\n"
    output += "\\end{tiny}\n"
    return output


def tests_to_csv(tests: dict, pps: int, test_count: int, val: callable, include_avg=True):
    output = "Name," + ",".join([str(i)
                                for i in range(test_count)])
    if include_avg:
        output += ",Average"
    for testname in tests:
        output += "\n" + testname + ","
        val_cache = []
        for test in tests[testname]:
            if test["pps"] != pps:
                continue
            output += f"{val(test):.3f},"
            val_cache.append(val(test))
        if include_avg:
            val_cache.sort()
            output += f"{sum(val_cache[2:-2]) / (len(val_cache)-4):.3f}"
    return output


def main():
    tests = os.listdir(input_path)
    parsed = {}
    for test in tests:
        parsed[test] = parse_test(test)
    for i in [1900, 19000, 190000, 1700000]:
        print("PPS: " + str(i))
        print(
            tests_to_csv(
                parsed,
                i,
                15,
                lambda x: x["packet_loss"],
                # lambda x: x["standard_deviation"],
                # lambda x: x["latency"][1],
                # lambda x: x["latency"][2],
                # lambda x: x["anomaly_latency_max"][1] if x["anomaly_count"] > 0 else 0,
                # lambda x: x["anomaly_latency_avg"][1] if x["anomaly_count"] > 0 else 0,
                # lambda x: x["anomaly_duration_avg"] if x["anomaly_count"] > 0 else 0,
                False,
            )
        )
        print()


if __name__ == "__main__":
    main()
