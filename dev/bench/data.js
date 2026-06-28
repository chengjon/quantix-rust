window.BENCHMARK_DATA = {
  "lastUpdate": 1782629013821,
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
      }
    ]
  }
}