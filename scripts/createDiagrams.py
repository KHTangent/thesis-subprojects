#!/usr/bin/env python3
"""
This script is even more specific than the other one, and should probably not be used
"""

import os
from math import sqrt
import matplotlib.pyplot as plt
import parseTestSummaries as pts

plotting_value = lambda e: e["latency"][1]
plotting_pps = 190000

def extract_data(tests: list, pps: int, val: callable):
    data = []
    for test in tests:
        if test["pps"] == pps:
            data.append(val(test))
    return data

def main(): 
    tests = os.listdir(pts.input_path)
    datas = []
    for test in tests:
        parsed = pts.parse_test(test)
        data = extract_data(parsed, plotting_pps, plotting_value)
        avg = sum(data) / len(data)
        sd = sqrt(sum(map(lambda e: (e - avg)**2, data)) / len(data))
        datas.append({
            "name": map_name(test),
            "avg": avg,
            "err": sd / sqrt(len(data))
        })
    datas.sort(key=lambda e: e["avg"])
    names = list(map(lambda e: e["name"], datas))
    averages = list(map(lambda e: e["avg"], datas))
    errors = list(map(lambda e: e["err"], datas))
    plt.rcdefaults()
    fig, ax = plt.subplots()
    ypos = list(range(len(names)))
    ax.barh(ypos, averages, xerr=errors, align='center')
    ax.set_yticks(ypos, labels=names)
    ax.invert_yaxis()
    ax.set_xlabel("Latency (ms)")
    ax.set_title(f"Latency of {plotting_pps} PPS tests")
    plt.show()


def map_name(name: str) -> str:
    # Helper function because I named my tests poorly
    if name == "load-double":
        return "stock-default-double-load"
    if name == "rt-load-half-threaded":
        return "rt-threaded-half-load"
    if name == "rt-load-default-threaded":
        return "rt-threaded-default-load"
    if name == "rt-threaded-double-queue":
        return "rt-threaded-double-idle"
    if name == "rt-threaded-default":
        return "rt-threaded-default-idle"
    if name == "threaded-default":
        return "stock-threaded-default-idle"
    if name == "rt-threaded-half-queue":
        return "rt-threaded-half-idle"
    if name == "rt-load-half":
        return "rt-default-half-load"
    if name == "threaded-half-queue":
        return "stock-threaded-half-idle"
    if name == "stock-router":
        return "stock-default-default-idle"
    if name == "rt-half-queue":
        return "rt-default-half-idle"
    if name == "stock-double-queue":
        return "stock-default-double-idle"
    if name == "load-default-threaded":
        return "stock-threaded-default-load"
    if name == "rt-stock":
        return "rt-default-default-idle"
    if name == "rt-load-double-threaded":
        return "rt-threaded-double-load"
    if name == "rt-double-queue":
        return "rt-default-double-idle"
    if name == "load-half":
        return "stock-default-half-load"
    if name == "stock-half-queue":
        return "stock-default-half-idle"
    if name == "load-half-threaded":
        return "stock-threaded-half-load"
    if name == "load-default":
        return "stock-default-default-load"
    if name == "rt-load":
        return "rt-default-default-load"
    if name == "load-double-threaded":
        return "stock-threaded-double-load"
    if name == "rt-load-double":
        return "rt-default-double-load"
    if name == "threaded-double-queue":
        return "stock-threaded-double-idle"
    else:
        raise Exception("Unknown test name: " + name)



if __name__ == "__main__":
    main()
