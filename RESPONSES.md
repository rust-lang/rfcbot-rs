# API Responses

An example request to the summary API (with the JSON prettified and truncated greatly):

```bash
$ time curl http://localhost:8080/summary

# JSON output below

real	0m0.424s
user	0m0.007s
sys	0m0.003s
```

```json
{
  "pull_requests": {
    "opened_per_day": {
      "2016-04-26": 15,
      "2016-04-27": 9,
      "2016-04-28": 10
    },
    "closed_per_day": {
      "2016-04-26": 17,
      "2016-04-27": 3,
      "2016-04-28": 19
    },
    "merged_per_day": {
      "2016-04-26": 15,
      "2016-04-27": 3,
      "2016-04-28": 16
    },
    "num_closed_per_week": {
      "2016-04-23": 44,
      "2016-04-30": 46
    },
    "days_open_before_close": {
      "2016-04-23": 10.208726851851852,
      "2016-04-30": 8.815974989935588
    },
    "current_open_age_days_mean": 25.63799043495164,
    "bors_retries": {
      "27807": 3,
      "29498": 2,
      "29732": 1
    }
  },
  "issues": {
    "opened_per_day": {
      "2016-04-26": 20,
      "2016-04-27": 20,
      "2016-04-28": 18
    },
    "closed_per_day": {
      "2016-04-26": 27,
      "2016-04-27": 9,
      "2016-04-28": 24
    },
    "num_closed_per_week": {
      "2016-04-23": 86,
      "2016-04-30": 74
    },
    "days_open_before_close": {
      "2016-04-23": 49.046675010766585,
      "2016-04-30": 41.58589245495495
    },
    "current_open_age_days_mean": 300.4759921378626,
    "num_open_p_high_issues": 63,
    "num_open_regression_nightly_issues": 71,
    "num_open_regression_beta_issues": 27,
    "num_open_regression_stable_issues": 5
  },
  "buildbots": {
    "per_builder_times_mins": {
      "auto-linux-32-nopt-t": {
        "2016-04-26": 95.50185185185185,
        "2016-04-27": 118.24444444444444,
        "2016-04-28": 92.30208333333333
      },
      "auto-linux-32-opt": {
        "2016-04-26": 104.89074074074074,
        "2016-04-27": 112.86666666666666,
        "2016-04-28": 101.13541666666667
      },
      "auto-linux-32cross-opt": {
        "2016-04-26": 68.99074074074075,
        "2016-04-27": 68.61481481481482,
        "2016-04-28": 65.30208333333333
      },
      "auto-linux-64-cargotest": {
        "2016-04-26": 57.77777777777778,
        "2016-04-27": 57.7,
        "2016-04-28": 58.022222222222226
      }
    },
    "per_builder_failures": {
      "auto-bitrig-64-opt": {
        "2016-04-26": 15,
        "2016-04-27": 16,
        "2016-04-28": 10
      },
      "auto-dragonflybsd-64-opt": {
        "2016-04-26": 15,
        "2016-04-27": 16,
        "2016-04-28": 10
      },
      "auto-freebsd10_32-1": {
        "2016-04-26": 15,
        "2016-04-27": 16,
        "2016-04-28": 10
      },
      "auto-freebsd10_64-1": {
        "2016-04-26": 16,
        "2016-04-27": 16,
        "2016-04-28": 10
      }
    }
  }
}
```
