#!/usr/bin/env python3
"""
This script is even more specific than the other one, and should probably not be used
"""

import os
from sys import argv
from math import sqrt
import matplotlib.pyplot as plt
import parseTestSummaries as pts


def plotting_value(e):
    if e["anomaly_count"] == 0:
        return None
    else:
        return e["anomaly_duration_avg"]


plotting_pps = 1900
plot_title = "Average anomaly duration of 1900 pps tests, n=2, t=250µs"
plot_xlabel = "Average anomaly duration (packets)"
logarithmic = False
include_error = True

color_options = ["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728"]


def extract_data(tests: list, pps: int, val: callable):
    data = []
    for test in tests:
        if test["pps"] == pps:
            v = val(test)
            if v is not None:
                data.append(val(test))
    return data


def main():
    tests = os.listdir(pts.input_path)
    datas = []
    for test in tests:
        parsed = pts.parse_test(test)
        data = extract_data(parsed, plotting_pps, plotting_value)
        if len(data) == 0:
            datas.append({
                "name": map_name(test),
                "avg": 0,
                "err": 0
            })
            continue
        avg = sum(data) / len(data)
        sd = sqrt(sum(map(lambda e: (e - avg)**2, data)) / len(data))
        datas.append({
            "name": map_name(test),
            "avg": avg,
            "err": sd / 2  # Divide by 2 since Matplotlib uses ±error
        })
    # datas.sort(key=lambda e: e["avg"])
    datas.sort(key=lambda e: e["name"])
    names = list(map(lambda e: e["name"], datas))
    averages = list(map(lambda e: e["avg"], datas))
    errors_up = list(map(lambda e: e["err"], datas))
    errors_down = []
    for i in range(len(errors_up)):
        errors_down.append(min(averages[i], errors_up[i]))
    colors = [color_options[i // 6] for i in range(24)]
    plt.rcdefaults()
    fig, ax = plt.subplots(figsize=(17, 10))
    ypos = list(range(len(names)))
    if include_error:
        ax.barh(ypos, averages, xerr=(errors_down, errors_up),
                align='center', color=colors)
    else:
        ax.barh(ypos, averages, align='center', color=colors)
    ax.set_yticks(ypos, labels=names)
    if logarithmic:
        ax.set_xscale('log')
    ax.invert_yaxis()
    ax.set_xlabel(plot_xlabel)
    ax.set_title(plot_title)
    if len(argv) > 1:
        plt.savefig(argv[1], dpi=150)
    else:
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
