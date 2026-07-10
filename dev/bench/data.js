window.BENCHMARK_DATA = {
  "lastUpdate": 1783665395680,
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
      },
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
          "id": "a1c763450f544e23461e7936217fdec6d395be19",
          "message": "Merge pull request #224 from chengjon/docs/refresh-gitnexus-metadata-20260613\n\ndocs: refresh gitnexus metadata",
          "timestamp": "2026-06-12T19:01:31Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/a1c763450f544e23461e7936217fdec6d395be19"
        },
        "date": 1781332935709,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 383616.2909899749,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3063451.50595172,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 40039542.15111905,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1299851.0613095239,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9782407.054367168,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 94011510.08742063,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1380864.80135101,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10331098.707429456,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 102683606.05722222,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17462.609934717904,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 171026.1833147582,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1923544.4788630963,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17744.95837848013,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 172617.42014637895,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1933608.3076719143,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 43806.97244649004,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 423021.35440049873,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4635817.434999999,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24719.758760567354,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 284342.77637473104,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3431349.1330517326,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42729.37237976494,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 662709.8585470708,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7465022.771428571,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6586.321149960068,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 75686.59835414305,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1327657.212042886,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7801.72642055393,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 94449.5531534424,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47857.81628686708,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18811.214679274006,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 156603.73379583526,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80330.06552170838,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 54.298675420033184,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 51.88018647436814,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.97137788192549,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1104.0661795337949,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11057.85920539222,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109379.44696933027,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 10944.493291305476,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 110353.84172686384,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1118523.3855717147,
            "unit": "ns"
          }
        ]
      },
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
          "id": "7ed44a86195ab840b634c01f227f5bdf4b42d618",
          "message": "Merge pull request #225 from chengjon/fix/security-audit-postgres-protocol-20260613\n\nfix: resolve postgres-protocol security audit",
          "timestamp": "2026-06-13T13:12:01Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/7ed44a86195ab840b634c01f227f5bdf4b42d618"
        },
        "date": 1781421213505,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 381974.1728390902,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3251756.3494479875,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39304399.40947619,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1306504.8345872436,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9846298.770200502,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 94410046.097877,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1447360.2355590477,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10707827.025130719,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 106615076.9896627,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17782.10469544612,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 175805.21159138207,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1961114.88680951,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17590.70456737111,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 174920.66980820737,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1979042.029714705,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 44950.613412890074,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 429892.8570764092,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4717004.959090909,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24612.67267425382,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 272707.52376262116,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3505065.1662169094,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42152.14986053951,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 652034.7622748296,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7342565.372142857,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6527.324296088119,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74234.24517934438,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1299278.6220170192,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7806.15828440972,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93786.19915020661,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47588.14066933848,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18994.58531379513,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 156648.23212577222,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80681.35457213556,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.534412321192605,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 52.013992087992186,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.47937623897498,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1109.1178671356881,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11072.198257970582,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 110486.42213355759,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11234.424447301606,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 113945.0872740041,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1148001.120025748,
            "unit": "ns"
          }
        ]
      },
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
          "id": "056e7e1add50f47cfeb01d30f9f3881a4dc318c6",
          "message": "Merge pull request #227 from chengjon/docs/workflow-function-tree-summary-20260615\n\ndocs: summarize workflow closure function tree status",
          "timestamp": "2026-06-15T03:03:36Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/056e7e1add50f47cfeb01d30f9f3881a4dc318c6"
        },
        "date": 1781509621674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 401059.7453190441,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3837246.0085267858,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 44085523.652904764,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1385438.660679413,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 10594638.684054233,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 101511244.27819446,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1469868.8994338838,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 11196666.062341271,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 106971145.11720237,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17338.227329183555,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 176537.7240394945,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1980208.9367916363,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 18776.11732796594,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 177002.20709062697,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 2011752.2839628821,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 45055.776724506926,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 430366.19641647534,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4763045.308095234,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 23599.68305976246,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 288060.0112265638,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3382673.360838907,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42243.78314187414,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 646294.6162197262,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7331679.212142854,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6525.433160558418,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74644.20631301362,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1304880.3728621062,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7871.050210226615,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93781.86549083769,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47625.29548483618,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18965.17086950629,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 157165.34350099886,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80746.71253560438,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.36969369762255,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 51.00005938207492,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.69688394460131,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1102.5133399268625,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11040.858978427397,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109307.59857940552,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11098.284408297723,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 112875.89272634081,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1148726.3655795464,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "6eba7ca9fa8422679ff37be49333119e42e64811",
          "message": "docs: refresh GitNexus metadata after paper closure\n\nRefresh GitNexus stats after paper query/cancel closure.",
          "timestamp": "2026-06-16T07:48:10Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/6eba7ca9fa8422679ff37be49333119e42e64811"
        },
        "date": 1781596348547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 364204.63335555553,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3006280.8535385877,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39240373.08611111,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1292401.7384464708,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9460703.514611112,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91089861.71956348,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1457843.0635352104,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 11086701.446846407,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 105551288.81357142,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14820.332156436756,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157180.65627593722,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1647395.864545499,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14772.65195763241,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157500.52687764907,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1661140.6665777788,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38667.35871497486,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 392616.89525690203,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 3999816.6807692326,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 25616.551382670546,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 321699.8200930103,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3324279.8547388683,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38581.32481370618,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 641706.5521815221,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6665953.676666666,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6187.689233996321,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71497.09674136262,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1214737.3206374121,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8037.922905632586,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95406.41341796143,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48641.7094062895,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16870.126974831244,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 149340.69604453625,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74674.53917371033,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.5546900313828,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.507598982633155,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.87696235973696,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1223.3150774121857,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12410.032089207569,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124584.16463124505,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9042.72427446829,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91168.59713584818,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 915573.2059674307,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2c1f4eac5d8aa008b8228ea17306270fc8719dc1",
          "message": "feat: add execution mode risk notices (#235)\n\n* feat: add execution mode risk notices\n\n* chore: close execution mode risk notice node\n\n* docs: refresh GitNexus metadata after risk notices",
          "timestamp": "2026-06-17T04:03:45Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2c1f4eac5d8aa008b8228ea17306270fc8719dc1"
        },
        "date": 1781681755887,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 358741.81935640855,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3061890.2918498674,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39437674.306539685,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1291391.5879437737,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9495121.331664577,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90380953.03822751,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1415103.6574040237,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10734056.75263072,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 104020543.00248016,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14826.719933455272,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157084.02656115367,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1647076.5836485606,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14862.28924530313,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157147.06848665775,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1663777.35007269,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38706.77817449099,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 393340.8413005458,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4010044.1588000003,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26751.456787750354,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 327421.75135711173,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3355929.6291306927,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38531.89903766206,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 640232.3128085112,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6684051.991333335,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6207.614623607871,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71744.00067587305,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1217557.1506837406,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8038.57352366656,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 94676.67293040187,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48461.195168451035,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16899.55685487724,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147023.92400318687,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74802.87405721092,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.481302591360944,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.55341323199371,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.30732421477825,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1226.9740283473218,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12447.740033870165,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124124.24887239159,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9050.102690497313,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91671.22901640434,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 920687.3090308647,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "958d710e9183b73b62f88690328451d87391f7e1",
          "message": "docs: propose qmt live capability identity hardening\n\n* docs: propose qmt live capability identity hardening\n\n* chore: close qmt live hardening design node",
          "timestamp": "2026-06-18T02:28:36Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/958d710e9183b73b62f88690328451d87391f7e1"
        },
        "date": 1781767093352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 381494.5336285449,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3484414.7351916744,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39933038.69623016,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1281117.0050217975,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9828866.406215541,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93685957.11539683,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1433881.4953124612,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10807985.171715686,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 107308839.35230158,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17813.626581509485,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 174659.61543316807,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1929626.6454412937,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 18030.42947793835,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 175605.27826579407,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1957194.5033277103,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 44197.374763833155,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 428082.2308184953,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4667415.484545455,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24649.430790098962,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 271873.3148728835,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3291337.2807263904,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42226.44185382497,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 643799.5740017664,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7340879.74428571,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6532.962512085925,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74138.63226390745,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1300566.5535259526,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7778.9068721658605,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93252.60302665312,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47633.33165214437,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 19082.38713724392,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 157371.10853276058,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80668.6941136918,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.51078980671708,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 52.01217161150684,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.49995809541694,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1099.9375836038253,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11033.265087755073,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109129.26736371318,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11107.737769129257,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 112031.28643175153,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1134785.6045950493,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2eadf409bf82939243b29782efe2c35af9b722e6",
          "message": "docs: note p0.3d graphiti backfill (#246)",
          "timestamp": "2026-06-19T02:04:32Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2eadf409bf82939243b29782efe2c35af9b722e6"
        },
        "date": 1781854356203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 383355.61871771095,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3707805.7540492387,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 47054813.57235119,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1289126.03607087,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9681162.033368839,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93571483.41488095,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1441155.5733824638,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10742762.930753967,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 107351240.52960317,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17367.05644012218,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 172000.36730142546,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1913368.0032444657,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17125.392458204326,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 170882.8428573109,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1948412.3378311277,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 44085.39584685982,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 422549.1332250644,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4628587.754090909,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 23813.888804684066,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 284294.4088186188,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3312258.9329511053,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42154.64578411929,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 639043.0079982711,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7345283.042857143,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6537.638571650367,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74297.33977769825,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1299261.7152761065,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7777.152662894573,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93790.00566546289,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47635.07916653536,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18994.081129673697,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 156963.19879316582,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80566.49196290855,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 55.60323547784103,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 52.32157030948551,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.38119793849486,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1101.2577077490637,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11003.761077658783,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109245.68211495336,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11225.62727026283,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 113012.67660509126,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1145240.0461937594,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2eadf409bf82939243b29782efe2c35af9b722e6",
          "message": "docs: note p0.3d graphiti backfill (#246)",
          "timestamp": "2026-06-19T02:04:32Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2eadf409bf82939243b29782efe2c35af9b722e6"
        },
        "date": 1781937991941,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 463027.98872157594,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 4711231.658679563,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 70957871.02814814,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1167397.6576597479,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9371796.328517856,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90866758.32269843,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1235105.5692348462,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 9957425.316309525,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 103110740.37037697,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14768.782996801208,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 155853.88545065295,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1700049.3640232598,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14743.574000862196,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 155891.9019300294,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1713456.0001249593,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 36225.235480397736,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 391827.30019880185,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4050234.2111999993,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26267.200488449147,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 333586.65252905316,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3605161.0985889803,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 40159.89797806221,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 687547.6828330435,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7188315.434285714,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 5538.34461815104,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 67975.66750131286,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1245168.3729863218,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 6717.470082432301,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 83284.31275623692,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 41555.25087282041,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 17047.649670827097,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147945.435036281,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74115.78201661803,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 47.162680980170016,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 44.900395604406775,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 68.38102749652404,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1160.2459768380734,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11661.314135655777,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 116207.07622582061,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 7761.1558483570025,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 79807.0416371975,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 793022.1352559542,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2eadf409bf82939243b29782efe2c35af9b722e6",
          "message": "docs: note p0.3d graphiti backfill (#246)",
          "timestamp": "2026-06-19T02:04:32Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2eadf409bf82939243b29782efe2c35af9b722e6"
        },
        "date": 1782026221568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 382383.0182965504,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3483235.0609621145,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39922093.180952385,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1305329.4315412468,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9698774.409933168,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93410653.26130953,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1444431.2867104113,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10979250.915319797,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 107601845.23668651,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17349.654265336398,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 173007.5325597401,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1917557.0826667973,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17109.8544064841,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 170351.36984497646,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1950331.8275885554,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 43409.32387235351,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 422984.2515115813,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4667887.32318182,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24677.35273810182,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 273500.4281321448,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3424577.0575285377,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42195.115516908256,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 641932.4984774931,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7354355.912857141,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6536.1743911380445,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74134.24635417601,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1302566.2155535633,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7769.361582173817,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93538.2241220427,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47601.26071886086,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18812.60526611723,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 157078.38819321987,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80717.90806943088,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 56.08122187661979,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 51.99323940013255,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 78.94666230482548,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1101.9623033856635,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11125.206989842085,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109424.18563416012,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11305.07829860293,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 113673.27188086807,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1149727.6741753977,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b3ec7487eb9a9d31998c3dc54b18d61f6b385f64",
          "message": "docs: update status registry and changelog (#266)",
          "timestamp": "2026-06-22T03:33:05Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/b3ec7487eb9a9d31998c3dc54b18d61f6b385f64"
        },
        "date": 1782114623942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 362215.24596547696,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3040612.8592387564,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39437775.08776984,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1305057.596367137,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9490464.412460318,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90779082.8651455,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1426934.6540206473,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10558165.221168429,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 105198327.76446429,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14788.91599664298,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157327.9432487687,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1648869.4609100034,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14829.710631705055,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 156835.06071629925,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1661803.2102415168,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38396.01921943556,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 392491.61791456194,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4012622.3507999997,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26982.97418733771,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 322140.8384578977,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3341854.5782930823,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 39209.59335338048,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 644895.360124696,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6712366.009333331,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6165.3134529305225,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71510.16723349593,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1215270.0165736096,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8079.325955260624,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95705.65574906452,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48631.31852328308,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16873.589245081603,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147192.99013963068,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74353.75627541648,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 54.2793657271661,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.55657673084739,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.88096798237474,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1226.732548804031,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12433.203874870585,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124273.60755468016,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9035.914576741474,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91695.15201560756,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 920660.1825543228,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f7f0abf3656d45720fdc7840a9848f1d9e1f42ee",
          "message": "test: add qmt live kill switch acceptance (#272)",
          "timestamp": "2026-06-23T04:38:12Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/f7f0abf3656d45720fdc7840a9848f1d9e1f42ee"
        },
        "date": 1782196351277,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 369613.0239270483,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3190477.915850738,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39246349.117761895,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1315315.5438380723,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9423977.465333333,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90824901.06346561,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1427090.9213290552,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10519006.45721781,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 104915348.95674604,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14797.643248581633,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157799.67009361973,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1654281.619710301,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14839.097568376572,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157534.51303987775,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1667856.5531634516,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38356.2425853966,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 395231.35705906095,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4025185.5079999994,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26530.166705598578,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 315014.05618676404,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3561564.421889545,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38642.83994732115,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 636710.5106171595,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6681703.5726666665,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6139.549804922284,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71419.1479877029,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1212994.2455015585,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8082.890980720371,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95663.31535717798,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48855.02657931357,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16793.30153750443,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147612.88816239906,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74717.66662211632,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 54.17887509295564,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.11130650832252,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.96970083867595,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1224.84817418886,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12427.673146514968,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124778.5122487096,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9105.222022659693,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92136.6050886877,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 924780.4889754247,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "321083ad1831382cfe02b29eeeb8ec4d94ab6b66",
          "message": "docs: add qmt live graphiti backfill record (#282)\n\n* docs: add qmt live graphiti backfill record\n\n* chore: close qmt live graphiti backfill node",
          "timestamp": "2026-06-24T04:49:15Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/321083ad1831382cfe02b29eeeb8ec4d94ab6b66"
        },
        "date": 1782282713926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 357346.78530709137,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3328347.939345599,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 38372357.6750873,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1267568.565664058,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9316492.602644842,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 89757046.23785713,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1403857.4509369999,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10410040.217403,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 103497879.86001983,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14845.308580945397,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157421.36207801683,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1653214.6796151912,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14912.257886828545,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157647.46492348268,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1667107.6369823604,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38668.63894704343,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 394642.12109651713,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4020188.596799999,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26765.1215843316,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 320049.6022459587,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3437520.7938728835,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38451.42479663392,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 640024.3846056806,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6670126.822666666,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6159.877062960874,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71409.04677830276,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1212631.7896451347,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8036.542176981343,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 94697.1324227658,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48266.32594547683,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16940.58989837429,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147528.29644993282,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74864.77762873699,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.48611277337702,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.60682015878565,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.86798403075935,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1224.5607640500157,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12428.666775953066,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124589.95782456924,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9056.534570337326,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91679.73622300058,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 918969.0358799574,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f0cf8b4d847357c8d4c748ddc3c5d6e9384e4e61",
          "message": "docs: close qmt live runtime readiness (#288)\n\n* docs: record qmt live readiness decision\n\n* chore: close qmt live readiness decision",
          "timestamp": "2026-06-24T16:26:04Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/f0cf8b4d847357c8d4c748ddc3c5d6e9384e4e61"
        },
        "date": 1782369177394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 381342.6305546385,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3128921.292605596,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39775810.73719841,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1286477.9866554884,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9613465.060578529,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93036452.36176588,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1439830.2707064743,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10845588.331349205,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 106233102.95242064,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17154.032805229202,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 170131.74820847332,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1906897.4338538032,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 17342.82136549081,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 169682.79550807792,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1930178.383263916,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 43457.223107113015,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 417817.11451132543,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4622717.22590909,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24663.208925821753,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 279376.9581800508,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3401209.9638050366,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42213.1372606222,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 640249.206426358,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7340242.183571428,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6534.585915538095,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74169.73886522911,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1299104.9213224757,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7774.624303006006,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93556.70429265466,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47494.976085471666,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18811.560837144192,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 156673.58903182083,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80468.9903260956,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.853966973770994,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 52.2868523169294,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.43267536215042,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1111.759965877902,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11036.512632517726,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109244.56430872003,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11314.520702261334,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 114047.64778365892,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1172392.9833735526,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "eb83a4a208a46a7acea533566832f55908e39fbb",
          "message": "docs: add openstock p0.8b graphiti backfill (#302)",
          "timestamp": "2026-06-26T05:43:23Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/eb83a4a208a46a7acea533566832f55908e39fbb"
        },
        "date": 1782455787452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 364253.70887775545,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3040733.578637566,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 40121267.75899206,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1288292.3152869132,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9652949.958297826,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90737653.01817462,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1428958.1600460927,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10659262.765187388,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 105654339.8130754,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14772.474507008494,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157155.83637772006,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1648339.1608512772,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14842.685140429245,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 156568.60653997434,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1661091.1045310518,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38607.547840968684,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 393204.7966376324,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4022849.5928000007,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26666.202234855285,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 313583.3240867785,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3436594.7512048376,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38494.40998983249,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 641314.1488546842,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6665809.706,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6187.160094174495,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71578.48079230302,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1217185.8545096423,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8083.248694555053,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95236.27362071183,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48697.39441046369,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16781.878226980683,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147625.1304918357,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74753.34404262109,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.6754140613113,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.576729925255385,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.16203743544051,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1227.3328047973866,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12410.873371266522,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 125265.28327625258,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9065.406278700637,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91616.62470099024,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 922294.5210971939,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2cd55ef0cd200362610a11071ab5bee25c9d3705",
          "message": "docs: add openstock p0.8d graphiti backfill (#308)",
          "timestamp": "2026-06-27T01:07:12Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2cd55ef0cd200362610a11071ab5bee25c9d3705"
        },
        "date": 1782541085552,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 359788.03421758604,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3044015.684857805,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39314124.4672143,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1300323.873257064,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9523730.099188492,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91467806.0593254,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1409938.665325766,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10391917.22989418,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 103900671.43892857,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14756.476252776501,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157417.25973465893,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1647323.4357912063,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14931.782014437253,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 156535.00536433284,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1661215.7678804053,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38214.7153848117,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 391015.0569036531,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4010863.5464000017,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 25300.057758778632,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 325624.47750751855,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3491846.5641445736,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38561.74448563859,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 639712.1125303432,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6680440.849333333,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6162.416306760466,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71739.06722340104,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1234255.272425232,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8086.200268502884,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95191.27972647472,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48512.36283598959,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16804.93578950204,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 150214.84326633712,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 75013.63978313825,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.79264421727421,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.54910555945905,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.38790302898956,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1222.1877861395462,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12447.788577799462,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124378.11685541723,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9053.419521342976,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91707.27987361446,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 921198.8277994178,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "45c03127f76720193ab7b70b906daaee8c588c0d",
          "message": "docs: add openstock p0.8e graphiti backfill (#312)",
          "timestamp": "2026-06-28T06:33:08Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/45c03127f76720193ab7b70b906daaee8c588c0d"
        },
        "date": 1782629012693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 359329.07582574117,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 2983632.095798771,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 38905691.03388095,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1295483.6169393787,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9467985.432807017,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90524579.31761906,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1420916.8987344655,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10439004.629583333,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 104545889.9579365,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14771.490546041243,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 156355.23635145673,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1653704.2356039956,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14798.550142166174,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 156866.1208281465,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1667927.2701667377,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38401.09669021065,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 392791.3040543972,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4013529.6848000004,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26495.270023571524,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 324166.1180635641,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3507574.708230504,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38545.35217272471,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 640175.493567713,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6692668.753999997,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6207.736634889737,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71787.06680555627,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1213715.3781339587,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8053.502577571855,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95065.02360872026,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48501.34079511691,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16808.96879832471,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147463.0541591905,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74840.72433491725,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.62767266877569,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.79930789239597,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.17599779107563,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1232.7395731997835,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12475.243465491485,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124672.0566156253,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9125.581909103308,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92650.20512625689,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 925091.8136608638,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "songjon",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "011967785e969b83836257bc433ef2e483134d10",
          "message": "docs: add openstock p0.8h analysis loop scope gate (#315)\n\nP0.8h extends P0.8d's narrow fixture→sma loop into a wider test-only\nslice: indicators (sma/ema/wma/bollinger/atr/obv/cci/williams_r) fan-out\nand MACrossStrategy signal-sequence validation against a new ~30-day\ncommitted fixture. Backtest path explicitly deferred — BacktestEngine\nhas no public run/feed method; requires separate production-code slice.\n\nCo-authored-by: Claude Opus 4.7 <noreply@anthropic.com>",
          "timestamp": "2026-06-29T05:02:42Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/011967785e969b83836257bc433ef2e483134d10"
        },
        "date": 1782717195470,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 364571.6691315444,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3292079.9691392495,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 38064395.15399206,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1256319.1941811712,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9410416.826902779,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 89705088.78074074,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1380090.215208409,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10430826.8274515,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 105278587.61363095,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14796.871364128914,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 156217.9944063044,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1652682.5387399686,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14745.861999554329,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 156994.07893877302,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1666291.8782705397,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38287.71447738758,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 393365.04340910504,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4012829.6935999985,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 25298.761813593028,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 321460.57739954244,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3404186.2910840143,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38542.45030013896,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 640564.9070936771,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6677491.950666667,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6192.48155346715,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71753.39404937062,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1214082.8052046997,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8043.524540449432,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 94850.11285682996,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48405.98749389732,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16934.085591263796,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 146983.04085722053,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74673.91677649153,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.59113699493318,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.76631334222221,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 75.89729219147956,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1243.5031843681127,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12414.436521990016,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 124121.9987663373,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9063.931679960833,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 91501.24521733653,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 921009.172168835,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "2571003c87fbe545ee189edd63238e356802046e",
          "message": "feat(openstock): add p0.9 consumer-side parsers + generic client skeleton\n\nAdds three fixture-driven parsers for OpenStock `/data/fetch` P0\ncategories (STOCK_CODES/ALL_STOCKS, TRADE_DATES/WORKDAYS, INDEX_KLINES)\nplus a generic `OpenStockClient` skeleton and uniform envelope types.\nNo live network in CI, no ClickHouse writes, no Kline/BacktestEngine\nchanges — pure additive slice enabling live wiring in a follow-up.\n\n- src/sources/openstock_envelope.rs: uniform success/error envelopes,\n  re-exports canonical `artifact_hash` from openstock_shadow (single\n  SHA-256 source of truth per CONNECTION_GUIDE §migration)\n- src/sources/openstock_client.rs: reqwest-based generic client with\n  envelope-aware `fetch<T>()` and three convenience wrappers\n- src/sources/openstock_codes.rs: STOCK_CODES + ALL_STOCKS parsers\n- src/sources/openstock_calendar.rs: TRADE_DATES + WORKDAYS parsers\n  (accepts both %Y-%m-%d and %Y%m%d date formats)\n- src/sources/openstock_index.rs: INDEX_KLINES parser reusing canonical\n  Kline with AdjustType::None; consumes pub(crate) normalize_symbol +\n  parse_live_time helpers (visibility-only widen, no signature change)\n- src/cli/commands/data.rs + handlers/app_shell.rs + handlers/mod.rs:\n  3 new ValidateCodes/ValidateCalendar/ValidateIndex subcommands wired\n  through dispatcher; sync fixture-only handlers, no network\n- 8 fixture files under tests/fixtures/openstock/ (positive + empty)\n- 4 integration test files under tests/ (flat, matching convention)\n- openspec/changes/openstock-data-consumption-p0-9/: proposal, tasks,\n  design (D1-D7 decisions documented), spec deltas with 6 requirements\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-30T05:03:19Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2571003c87fbe545ee189edd63238e356802046e"
        },
        "date": 1782801255485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 300019.0200749632,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 2896259.223956916,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 35968500.20215079,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1929463.9485098522,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 8563474.52438889,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 108609955.35422619,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 4797512.757035273,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 32153175.639511906,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 168121436.36873016,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 10938.655773559396,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 112767.14697904793,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1195791.191134575,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 11092.932101758108,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 113317.47155618733,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1209669.8590843708,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 26894.94890659115,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 274067.0676319724,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 2876848.62900295,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 16136.931439239961,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 185827.08897420697,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 2082205.9398451615,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 27935.27167839534,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 434119.43191708444,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 4977457.467499998,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 4015.6886299664393,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 44184.765474337895,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 847670.9002322466,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 4573.283618158079,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 60579.53377387752,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 30634.149519011797,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 12290.436917265219,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 106950.26272331683,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 54005.29486477352,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 31.090381704612227,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 29.294839592537848,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 46.30837669511207,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 626.320140914887,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 6282.074130116398,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 60224.28104164,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 4882.579954795728,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 49973.10323876542,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 499749.5978676136,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "8687d47e34f8ccd3c4c1a9dac6cf7339d82070d5",
          "message": "feat(openstock): WORKDAYS CLI surfaces full action matrix\n\nThe WORKDAYS category is action-driven in the runtime (eltdx provider\nsupports today/today_is_workday/is_workday/range/next_workday/\nprevious_workday). The previous CLI shape (`--year`) was based on the\nP0.9 union-calendar assumption and silently did nothing useful —\nruntime ignored the year param and returned `action=today` for every\ncall.\n\nReplaced the CLI subcommand shape:\n  fetch-workdays [--action <default today>]\n                 [--date YYYY-MM-DD]\n                 [--start YYYY-MM-DD] [--end YYYY-MM-DD]\n\n`OpenStockClient::fetch_workdays` signature changed accordingly; the\nhandler prints the full record shape (action/date/is_workday/\ntoday_is_workday/next_workday/previous_workday) so callers can see\nexactly what runtime returned. Live test updated to drive a\nconfigurable action via env vars.\n\nNote: live smoke shows runtime itself downgrades next_workday /\nprevious_workday to `action=today` — that is an upstream provider\nbehavior, not a CLI bug, and is left for the OpenStock runtime repo\nto address.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-30T15:15:54Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/8687d47e34f8ccd3c4c1a9dac6cf7339d82070d5"
        },
        "date": 1782889239760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 295582.46866598155,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 2900847.405652282,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 36813421.165246025,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 855113.2686298216,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 10098479.716795051,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 80612932.596627,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 2299571.0941674765,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 20153373.955122653,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 181477057.2408135,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 11138.045442315093,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 113253.25122115195,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1195227.5054630816,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 11289.548363654758,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 112105.37530400808,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1254667.8737927405,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 27360.25419919614,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 282060.1624946865,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 2929989.9252251745,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 15784.458266234535,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 191191.14708446362,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 2128208.603475987,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 28720.257481074565,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 442614.7705537132,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 4960099.240499998,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 3979.000316492851,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 44344.20137755135,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 840775.660171795,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 4692.809188020699,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 60045.72444427732,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 30047.022781400636,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 12392.867148968535,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 109199.16814886205,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 54370.701376921395,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 31.604106959500804,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 29.874685576601934,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 46.805326211673744,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 660.2029184455735,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 6381.106278291686,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 62963.09971862113,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 4965.381375594331,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 50688.7685862167,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 518374.9571991612,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "49d11369ffd8e20e1695364ae7c8a96961b06f99",
          "message": "chore(openstock): P0.12 follow-ups — full tdx-api removal\n\nExecutes 3 of the 4 non-blocking follow-ups from the P0.11 closeout\nreport (the 4th, tdx-api Docker image deprecation, is filed in the\nopenstock repo at docs/operations/TDX_API_IMAGE_DEPRECATION_2026-07-02.md).\n\n1. docker-compose.yml: full removal of tdx-api service block + volume\n   (was commented for rollback safety in P0.11c Phase 5). Compose\n   config validates clean.\n2. scripts/daily-update.sh: rewritten to use OpenStock endpoints\n   (data openstock fetch-calendar + data import-klines per-code).\n   Drops --all batch flag (no equivalent yet; codes passed as args).\n   Requires OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY env. APPLY=1\n   passes --apply (still gated by QUANTIX_OPENSTOCK_KLINE_APPLY=yes).\n3. docs/CLI_COMMAND_MANUAL.html: full cleanup — removed 18 nav\n   entries and 19 detail sections under cmd-data-tdx-api; replaced\n   with a single compact deprecation section pointing at top-level\n   replacements (data import-ticks/import-klines/openstock fetch-*).\n   Down from 152 refs to 4 intentional deprecation markers.\n\nVerification:\n- docker compose -f docker-compose.yml config (validates clean)\n- bash -n scripts/daily-update.sh\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-01T17:32:11Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/49d11369ffd8e20e1695364ae7c8a96961b06f99"
        },
        "date": 1782973904713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 359705.2060662801,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3203727.9366193265,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39602192.528825395,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1350973.4342842866,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9631390.472212302,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 92128003.54797618,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1394346.218490246,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10098870.968437761,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 101736895.67646827,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14861.931334785304,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157650.26909937852,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1658318.7943488394,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14928.595161903693,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157645.04572722185,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1665799.2255036351,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38578.49588372378,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 395112.7663841627,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4018916.7260000007,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 26145.98078111815,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 341213.66644783656,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3621108.7409443557,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38824.65482676802,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 642104.2565238872,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6691792.121333334,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6206.474145130929,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71884.11410256164,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1217734.963068686,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8160.193152930205,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95496.01234267557,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 49138.591612436765,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16865.89963748921,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147428.16309573237,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74712.14190074544,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 52.87599905168985,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.50022278754663,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.12710925425863,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1255.2235048298385,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12768.545131735415,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127508.87634527139,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9116.123787566175,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92855.80966656742,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 935744.4539760561,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "f859deae8e0b3c40496ac44600066bfe02690db3",
          "message": "chore(governance): mark P0.13b-2 card as completed\n\nP0.13b-2 implementation merged: MinuteShare struct, fetch_minute_share\nclient method (envelope path with retry/circuit breaker), parse_minute_share\nparser with INV-2C skip semantics, FetchMinuteShare CLI subcommand, live\ntests, OpenSpec change archived as 2026-07-03-openstock-data-consumption-p0-13b-2.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-03T00:38:18Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/f859deae8e0b3c40496ac44600066bfe02690db3"
        },
        "date": 1783059993146,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 369539.1246820606,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3446588.546961665,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39887078.40951587,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1330311.1733114496,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9545537.580900794,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91371655.7961111,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1414247.5470091575,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10164585.54859127,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 101927193.09380952,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 15040.693279243806,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157856.99878819182,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1664083.986347974,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14939.364820198436,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157723.2950691488,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1674085.830784603,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38709.6629484303,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 394775.7923020636,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4046514.9819999994,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27470.69404220122,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 315763.9920841082,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3362159.515712689,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 39127.50535242807,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 643211.6401360702,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6712181.701333336,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6269.074305599573,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71772.21582523327,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1233248.2283123755,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8101.604685720641,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95596.77286242325,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48856.832685871355,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16803.448281564903,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147454.60951842065,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74915.27915583337,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 52.8722957775071,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.469726145876265,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.43525443770292,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1263.452716223905,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12801.786994328397,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127905.13147318563,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9158.840402079757,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92578.46728241352,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 939171.6681304538,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "f859deae8e0b3c40496ac44600066bfe02690db3",
          "message": "chore(governance): mark P0.13b-2 card as completed\n\nP0.13b-2 implementation merged: MinuteShare struct, fetch_minute_share\nclient method (envelope path with retry/circuit breaker), parse_minute_share\nparser with INV-2C skip semantics, FetchMinuteShare CLI subcommand, live\ntests, OpenSpec change archived as 2026-07-03-openstock-data-consumption-p0-13b-2.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-03T00:38:18Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/f859deae8e0b3c40496ac44600066bfe02690db3"
        },
        "date": 1783145929050,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 371400.85566041374,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3369238.8445811286,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39755152.030063495,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1339843.024256989,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9577342.175735172,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91119416.12938493,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1388938.6281051585,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10411877.665910494,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 102011088.27426587,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14860.941689145826,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 158972.83374816627,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1663952.6708062682,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14907.897146710386,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157936.13288830812,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1676362.5167735443,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38618.49602189031,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 394845.87988221727,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4043821.1500000013,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27118.0326130282,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 317197.4931651345,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3402725.2657657536,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38883.28491277176,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 642117.1149937981,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6720668.109333334,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6237.648062049568,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 72150.00667430529,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1222845.1369809867,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8091.9605539926015,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95792.23236222574,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48740.90503057003,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16805.166797199345,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147603.4834522594,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74404.79994489187,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.17569498278833,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.8942330162404,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.5794838781264,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1256.1937818363347,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12785.304593792085,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127932.51787731571,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9161.955016582775,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 93045.73290446667,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 938411.5807469125,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "2ea8ecfba0c5f756bcd169b15e147e46500a1f45",
          "message": "chore(openspec): archive openstock-data-consumption-p0-14\n\nMerge the 5 added requirements (REQ-PERSIST-001..005) into the main\nopenstock-data-consumption spec and move the change to\nopenspec/changes/archive/2026-07-05-openstock-data-consumption-p0-14/.\n\nP0.14 card is marked complete and the implementation is on master:\n- minute_klines/minute_shares tables and row types\n- stream_minute_*_to_clickhouse consumers\n- U1-U8 unit tests + L1/L2 live tests\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-05T02:26:16Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/2ea8ecfba0c5f756bcd169b15e147e46500a1f45"
        },
        "date": 1783233474935,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 358039.1497065992,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3059089.2696124343,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39471713.94522222,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1337263.426262255,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9601599.727727652,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90936716.95505951,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1388787.2162469935,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10234148.788247352,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 102322991.4889484,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 15206.049999592558,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 161219.68514706966,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1796417.6663947094,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 15890.739579716223,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 161698.88172686662,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1709796.8614307425,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38977.33434425909,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 425272.9880912315,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4265267.775833335,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27205.430754567595,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 328297.20399875107,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3404073.8659293354,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38858.40475575668,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 645039.3985738051,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6702219.160666671,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6208.23309104281,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 72862.63932771157,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1219258.3673928767,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8093.0799536459035,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95847.64419353752,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48493.08340614223,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16830.30020824094,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 146923.08546576533,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74308.12297022744,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 52.77212963208016,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.92284457660994,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.46468394836758,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1257.070604323022,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12754.689746862603,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127734.00507867403,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9111.083721995867,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92411.08382725186,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 935616.8368803522,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "94279a3886f821ee0f3bbac87c1376cf2bd5c6c1",
          "message": "docs(spec): apply P0.15a design review fixes (C1/M1/M2 + test matrix)\n\nAddress APPROVE_WITH_NOTES findings from design review:\n- C1: clarify ClickHouseMinuteKlineSink<'a> lifetime is inferred from\n  struct-literal field assignment (no turbofish needed); R1 updated\n  with the concrete pattern\n- M1: document inline `use crate::db::clickhouse::minute::{Sink, ...}`\n  import path for both handlers (sink structs are pub(crate) inside\n  the private minute module per P0.14 INV-4D)\n- M2: redesign compute_apply as env-aware (single-arg signature) so\n  U2/U3 must set QUANTIX_OPENSTOCK_MINUTE_APPLY via std::env::set_var\n  — exercises the real env-var name rather than testing && as a\n  tautology. Mirrors src/cli/tests/risk.rs:352-353 pattern\n- Add missing §6 Test matrix section (U1-U3 unit + L1/L2 live were\n  referenced by §5 file table but not actually defined)\n- Renumber §7-§12 (was §6-§11) to absorb the new test section\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-05T14:46:01Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/94279a3886f821ee0f3bbac87c1376cf2bd5c6c1"
        },
        "date": 1783321739462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 363819.90417111106,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3358240.232165224,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 43530916.889714286,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1306323.8818676302,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9653874.204847535,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91134604.9802381,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1404769.521464286,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10019722.381620718,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 100969016.59676588,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14922.713006864233,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157662.58715241327,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1659406.5470422963,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14935.362485354011,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 158111.7842203681,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1673488.885091933,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38719.9589520181,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 394368.5024234087,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4034549.5023999996,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27415.19824598287,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 324386.1064068359,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3480063.357087004,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38992.24375443438,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 644556.5575542673,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6693925.5,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6226.206172395551,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 72075.60741777811,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1226150.7841625083,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8109.999035130231,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95713.77001204761,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48947.52385283308,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16814.014891772054,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 146972.55672573563,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74811.99683842258,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.18945807398716,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.91882369723914,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.527924520359,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1254.5793424258898,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12845.366022656457,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 128058.6855824818,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9110.205588620343,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92720.41992182074,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 932993.3864385833,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "94279a3886f821ee0f3bbac87c1376cf2bd5c6c1",
          "message": "docs(spec): apply P0.15a design review fixes (C1/M1/M2 + test matrix)\n\nAddress APPROVE_WITH_NOTES findings from design review:\n- C1: clarify ClickHouseMinuteKlineSink<'a> lifetime is inferred from\n  struct-literal field assignment (no turbofish needed); R1 updated\n  with the concrete pattern\n- M1: document inline `use crate::db::clickhouse::minute::{Sink, ...}`\n  import path for both handlers (sink structs are pub(crate) inside\n  the private minute module per P0.14 INV-4D)\n- M2: redesign compute_apply as env-aware (single-arg signature) so\n  U2/U3 must set QUANTIX_OPENSTOCK_MINUTE_APPLY via std::env::set_var\n  — exercises the real env-var name rather than testing && as a\n  tautology. Mirrors src/cli/tests/risk.rs:352-353 pattern\n- Add missing §6 Test matrix section (U1-U3 unit + L1/L2 live were\n  referenced by §5 file table but not actually defined)\n- Renumber §7-§12 (was §6-§11) to absorb the new test section\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-05T14:46:01Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/94279a3886f821ee0f3bbac87c1376cf2bd5c6c1"
        },
        "date": 1783406497400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 359355.26694280404,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3373971.2792019406,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 38994677.96698413,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1310820.8960999772,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9535736.119152777,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91166515.66321427,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1383452.005353264,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10024055.34361993,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 100848309.1748611,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14937.763849335177,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157703.19197992445,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1657674.3589141932,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14895.933133528326,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157579.93693229032,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1667582.5291236546,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38880.62183010027,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 395313.22019613103,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4031850.0844,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27387.832263661432,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 338937.8560134438,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3481841.383598315,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38928.60948212052,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 641886.7571540971,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6743103.716666668,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6221.810785323337,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71983.24726060989,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1220002.7100601434,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8086.32960592488,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95291.0003614706,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48563.02866202562,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16842.114667490925,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 147098.75225559284,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74604.43360459582,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 53.16668051256385,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.77744433297682,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.51556935577634,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1257.6803129996438,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12762.76613919674,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127360.48621405233,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9246.861910463347,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 93224.89703778637,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 941731.768691437,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "94279a3886f821ee0f3bbac87c1376cf2bd5c6c1",
          "message": "docs(spec): apply P0.15a design review fixes (C1/M1/M2 + test matrix)\n\nAddress APPROVE_WITH_NOTES findings from design review:\n- C1: clarify ClickHouseMinuteKlineSink<'a> lifetime is inferred from\n  struct-literal field assignment (no turbofish needed); R1 updated\n  with the concrete pattern\n- M1: document inline `use crate::db::clickhouse::minute::{Sink, ...}`\n  import path for both handlers (sink structs are pub(crate) inside\n  the private minute module per P0.14 INV-4D)\n- M2: redesign compute_apply as env-aware (single-arg signature) so\n  U2/U3 must set QUANTIX_OPENSTOCK_MINUTE_APPLY via std::env::set_var\n  — exercises the real env-var name rather than testing && as a\n  tautology. Mirrors src/cli/tests/risk.rs:352-353 pattern\n- Add missing §6 Test matrix section (U1-U3 unit + L1/L2 live were\n  referenced by §5 file table but not actually defined)\n- Renumber §7-§12 (was §6-§11) to absorb the new test section\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-05T14:46:01Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/94279a3886f821ee0f3bbac87c1376cf2bd5c6c1"
        },
        "date": 1783490064158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 365328.36789086106,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3436758.4287795746,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 40003292.14663492,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1304159.7431062926,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9496192.260497075,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 90990115.40319446,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1393918.05985989,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10111585.682365522,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 101673981.1898611,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14843.549600200318,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 158056.81105435762,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1657439.4849347887,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 15015.11107206801,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157999.72277444942,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1666665.2260481291,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38695.503327216444,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 393031.9891214244,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4048247.7768000006,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 25977.522695523217,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 334429.5038076854,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3509020.333602347,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38842.152653589124,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 642347.0051295758,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6677346.484000001,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6201.855401855242,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71765.17109374772,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1218685.9380430877,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8125.650085579385,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95836.84082470882,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 49065.21980034569,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16800.652683845627,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 146705.8278219468,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74612.362768903,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 52.77464159033092,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.49995716452187,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.26456364030446,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1252.3738004336465,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12752.492815576685,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127350.61610278081,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9157.506823627158,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92871.96322285914,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 938768.7016386492,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "94279a3886f821ee0f3bbac87c1376cf2bd5c6c1",
          "message": "docs(spec): apply P0.15a design review fixes (C1/M1/M2 + test matrix)\n\nAddress APPROVE_WITH_NOTES findings from design review:\n- C1: clarify ClickHouseMinuteKlineSink<'a> lifetime is inferred from\n  struct-literal field assignment (no turbofish needed); R1 updated\n  with the concrete pattern\n- M1: document inline `use crate::db::clickhouse::minute::{Sink, ...}`\n  import path for both handlers (sink structs are pub(crate) inside\n  the private minute module per P0.14 INV-4D)\n- M2: redesign compute_apply as env-aware (single-arg signature) so\n  U2/U3 must set QUANTIX_OPENSTOCK_MINUTE_APPLY via std::env::set_var\n  — exercises the real env-var name rather than testing && as a\n  tautology. Mirrors src/cli/tests/risk.rs:352-353 pattern\n- Add missing §6 Test matrix section (U1-U3 unit + L1/L2 live were\n  referenced by §5 file table but not actually defined)\n- Renumber §7-§12 (was §6-§11) to absorb the new test section\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-05T14:46:01Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/94279a3886f821ee0f3bbac87c1376cf2bd5c6c1"
        },
        "date": 1783579059934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 365843.0231978242,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3049287.3468981483,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39957796.476865076,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1339561.9764964725,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9575269.850132937,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 91448352.62303571,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1395652.0797655398,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10056701.813953632,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 100615166.7203373,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 14944.366685973082,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 157917.32202945376,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1657584.86252045,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 14933.606498350242,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 157738.52751781236,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1677194.97347204,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 38603.71289005948,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 393680.1971092302,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4045882.202800001,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 27633.14218488394,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 313245.6510953953,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3457764.238918489,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 38805.28316929474,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 644436.7315872252,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 6711280.337333335,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6206.439691143393,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 71844.76036465187,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1219381.740454011,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 8099.186660129919,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 95558.5460180757,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 48718.22827370584,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 16993.766839921293,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 146624.9407773357,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 74536.63391908376,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 52.71385503986944,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 50.48280796057397,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 74.25870850350876,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1269.0662606895505,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 12712.806701019874,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 127469.47606975398,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 9116.130252032284,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 92501.75699171083,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 935570.9547477986,
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "committer": {
            "name": "JohnC",
            "username": "chengjon",
            "email": "ninjas@sina.com"
          },
          "id": "3ee41c416e85813c97bc6c6755b35eebe3f0ec12",
          "message": "docs(p0.15b-pre): add versioned DDL for stock_info + import_state\n\nP0.15b spec §7 defined the schema for quantix.stock_info and\nquantix.import_state, but the DDL only lived inside the spec doc —\noperators had no runnable file. Task 9 of P0.15b created the tables\nout-of-band on the NAS Postgres instance; this commit captures that\nDDL as a proper versioned artifact.\n\n- db/schema/quantix_openstock_import_init.sql: operator-runbook DDL\n  (same convention as quantix_shadow_init.sql — intentionally NOT\n  applied by automated CI migration; opt-in write path). Idempotent\n  (IF NOT EXISTS on every object). Verified clean-run + idempotent-run\n  on quantix_test database.\n\n- deploy/nas/quantix-openstock-import/README.md: deployment guide\n  pointing at the DDL file as a precondition, plus build/load/run\n  instructions for the NAS host.\n\nAll 4 live integration tests (T1-T4) re-verified green after applying\nthe new DDL file to a fresh quantix_test schema.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-07-09T05:50:23Z",
          "url": "https://github.com/chengjon/quantix-rust/commit/3ee41c416e85813c97bc6c6755b35eebe3f0ec12"
        },
        "date": 1783665395133,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "batch/process_in_batches/10000",
            "value": 382572.53161350545,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/100000",
            "value": 3189660.294212615,
            "unit": "ns"
          },
          {
            "name": "batch/process_in_batches/1000000",
            "value": 39748710.87943651,
            "unit": "ns"
          },
          {
            "name": "export/csv/1000",
            "value": 1300102.7682865646,
            "unit": "ns"
          },
          {
            "name": "export/csv/10000",
            "value": 9744494.21530284,
            "unit": "ns"
          },
          {
            "name": "export/csv/100000",
            "value": 93200234.7200397,
            "unit": "ns"
          },
          {
            "name": "export/json/1000",
            "value": 1436930.9588913887,
            "unit": "ns"
          },
          {
            "name": "export/json/10000",
            "value": 10784231.911760036,
            "unit": "ns"
          },
          {
            "name": "export/json/100000",
            "value": 107674992.69315477,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/100",
            "value": 17902.591688717905,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/1000",
            "value": 176317.6657999472,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_12/10000",
            "value": 1940872.9363476026,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/100",
            "value": 18080.876062658008,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/1000",
            "value": 176770.09658458695,
            "unit": "ns"
          },
          {
            "name": "indicators/ema_26/10000",
            "value": 1960867.4904718352,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/100",
            "value": 44308.274514942736,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/1000",
            "value": 428380.0532916364,
            "unit": "ns"
          },
          {
            "name": "indicators/macd/10000",
            "value": 4665099.31727273,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/100",
            "value": 24545.600369265252,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/1000",
            "value": 286069.53255801916,
            "unit": "ns"
          },
          {
            "name": "indicators/rsi_14/10000",
            "value": 3411623.2210685126,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/100",
            "value": 42071.218669350965,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/1000",
            "value": 641537.7288754284,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_20/10000",
            "value": 7361797.156428573,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/100",
            "value": 6623.811720808015,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/1000",
            "value": 74249.01562147956,
            "unit": "ns"
          },
          {
            "name": "indicators/sma_5/10000",
            "value": 1300420.2347361273,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/100",
            "value": 7757.700146610112,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/1000",
            "value": 93464.36512714061,
            "unit": "ns"
          },
          {
            "name": "performance/max_drawdown/500",
            "value": 47147.748686689796,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/100",
            "value": 18950.88720601683,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/1000",
            "value": 157313.91812108076,
            "unit": "ns"
          },
          {
            "name": "performance/sharpe_ratio/500",
            "value": 80775.71847794787,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/100",
            "value": 54.09517907939563,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/1000",
            "value": 53.03236463043969,
            "unit": "ns"
          },
          {
            "name": "performance/total_return/500",
            "value": 76.62942244117885,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/100",
            "value": 1108.4016484273138,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/1000",
            "value": 11002.363272772076,
            "unit": "ns"
          },
          {
            "name": "validation/quality_report/10000",
            "value": 109393.74380896226,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/100",
            "value": 11117.299728260661,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/1000",
            "value": 112133.46112277784,
            "unit": "ns"
          },
          {
            "name": "validation/validate_klines/10000",
            "value": 1131306.5229481193,
            "unit": "ns"
          }
        ]
      }
    ]
  }
}