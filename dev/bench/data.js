window.BENCHMARK_DATA = {
  "lastUpdate": 1781681756941,
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
      }
    ]
  }
}