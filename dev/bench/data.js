window.BENCHMARK_DATA = {
  "lastUpdate": 1781252892701,
  "repoUrl": "https://github.com/chengjon/quantix-rust",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "chengjon@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "078c5902067469523f5cd1c605cee80a950982d7",
          "message": "Merge pull request #222 from chengjon/fix/scheduled-benchmark-criterion-output-20260611\n\nFix scheduled benchmark Criterion result parsing",
          "timestamp": "2026-06-11T13:08:38Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/078c5902067469523f5cd1c605cee80a950982d7"
        },
        "date": 1781252892169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 381462.9815321777,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3121048.4763976326,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39702446.530357145,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1293769.4147012471,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9603656.632901002,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93013041.85873017,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1383754.6260464764,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10323309.99991402,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 102658390.19426587,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17896.704703318752,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 171159.0665255761,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1921547.7940463808,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17894.863040183518,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 170601.1857002357,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1936327.686782907,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 44580.17464391302,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 427481.3482778628,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4649914.645454545,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 23952.988258671343,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 299282.0583234035,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3485350.697755454,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42179.94363568097,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 648826.2751183495,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7345506.127857146,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6578.387963301397,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74192.75155401543,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1305768.5860733867,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7765.313705457329,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93276.4280872651,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47414.091882097244,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18797.90345779607,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 156966.7662934236,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80380.52195084577,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.329418550575035,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 52.17667161348133,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.91346867806232,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1103.595355728164,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11048.36557925738,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109721.58266285095,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 10967.815550411837,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 109841.33443284394,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1112196.0428968498,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}