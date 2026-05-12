use super::*;

pub async fn run_factor_command(cmd: FactorCommands) -> Result<()> {
    match cmd {
        FactorCommands::List { category, verbose } => {
            let catalog = crate::factor::builtin_factor_catalog();
            for meta in catalog.list() {
                if let Some(category_filter) = &category {
                    if format!("{:?}", meta.category).to_lowercase()
                        != category_filter.to_lowercase()
                    {
                        continue;
                    }
                }

                if verbose {
                    println!(
                        "{}\t{:?}\t{}\tfields={:?}\tmissing={:?}",
                        meta.id,
                        meta.category,
                        meta.description,
                        meta.required_fields,
                        meta.missing_policy
                    );
                } else {
                    println!("{}\t{:?}\t{}", meta.id, meta.category, meta.description);
                }
            }
            Ok(())
        }
        FactorCommands::Compute {
            input,
            factors,
            symbols,
            start,
            end,
            format,
            output,
            skip_checks,
        } => {
            let start = parse_factor_date(&start)?;
            let end = parse_factor_date(&end)?;
            let catalog = crate::factor::builtin_factor_catalog();
            let required_fields = factors
                .iter()
                .filter_map(|factor| {
                    catalog
                        .list()
                        .iter()
                        .find(|meta| meta.id == *factor)
                        .map(|meta| meta.required_fields.clone())
                })
                .flatten()
                .collect::<Vec<_>>();
            let request = crate::factor::FactorLoadRequest {
                symbols,
                start,
                end,
                required_fields,
            };
            let loader = crate::factor::CsvFactorDataLoader::new(input);
            let dataset = crate::factor::FactorDataset::from_loader(&loader, &request).await?;
            if !skip_checks {
                dataset.ensure_time_aligned()?;
                dataset.validate_no_lookahead_basic()?;
            }

            for factor in factors {
                let result = catalog.compute(&factor, &dataset)?;
                match format {
                    FactorOutputFormat::Table => {
                        println!("factor_id={}", result.factor_id);
                        println!("{}", result.frame);
                    }
                    FactorOutputFormat::Csv => {
                        let output = output.as_deref().ok_or_else(|| {
                            QuantixError::Config(
                                "factor csv output requires --output <path>".to_string(),
                            )
                        })?;
                        std::fs::write(
                            output,
                            crate::factor::factor_result_to_csv_string(&result)?,
                        )?;
                    }
                    FactorOutputFormat::Json => {
                        let output = output.as_deref().ok_or_else(|| {
                            QuantixError::Config(
                                "factor json output requires --output <path>".to_string(),
                            )
                        })?;
                        std::fs::write(
                            output,
                            crate::factor::factor_result_to_json_string(&result)?,
                        )?;
                    }
                    FactorOutputFormat::Parquet => {
                        let output = output.as_deref().ok_or_else(|| {
                            QuantixError::Config(
                                "factor parquet output requires --output <path>".to_string(),
                            )
                        })?;
                        crate::factor::factor_result_to_parquet_file(&result, output)?;
                    }
                }
            }
            Ok(())
        }
        FactorCommands::Evaluate {
            input,
            factor,
            symbols,
            start,
            end,
            horizon,
            format,
            output,
            skip_checks,
        } => {
            let start = parse_factor_date(&start)?;
            let end = parse_factor_date(&end)?;
            let catalog = crate::factor::builtin_factor_catalog();
            let mut required_fields = vec!["close".to_string()];
            if let Some(meta) = catalog.list().iter().find(|meta| meta.id == factor) {
                required_fields.extend(meta.required_fields.clone());
            }
            required_fields.sort();
            required_fields.dedup();

            let request = crate::factor::FactorLoadRequest {
                symbols,
                start,
                end,
                required_fields,
            };
            let loader = crate::factor::CsvFactorDataLoader::new(input);
            let dataset = crate::factor::FactorDataset::from_loader(&loader, &request).await?;
            if !skip_checks {
                dataset.ensure_time_aligned()?;
                dataset.validate_no_lookahead_basic()?;
            }

            let factor_result = catalog.compute(&factor, &dataset)?;
            let evaluation = crate::factor::evaluate_factor_ic(&dataset, &factor_result, horizon)?;
            match format {
                FactorOutputFormat::Table => {
                    println!(
                        "factor_id={}\thorizon={}\tobservations={}\tic_mean={:?}\tic_std={:?}\tir={:?}",
                        evaluation.summary.factor_id,
                        evaluation.summary.horizon,
                        evaluation.summary.observations,
                        evaluation.summary.ic_mean,
                        evaluation.summary.ic_std,
                        evaluation.summary.ir
                    );
                    println!("{}", evaluation.by_date);
                }
                FactorOutputFormat::Json => {
                    let json = crate::factor::factor_ic_result_to_json_string(&evaluation)?;
                    if let Some(output) = output {
                        std::fs::write(output, json)?;
                    } else {
                        println!("{}", json);
                    }
                }
                FactorOutputFormat::Csv => {
                    let output = output.as_deref().ok_or_else(|| {
                        QuantixError::Config(
                            "factor evaluate csv output requires --output <path>".to_string(),
                        )
                    })?;
                    std::fs::write(
                        output,
                        crate::factor::factor_ic_result_to_csv_string(&evaluation)?,
                    )?;
                }
                FactorOutputFormat::Parquet => {
                    return Err(QuantixError::Unsupported(
                        "factor evaluate parquet output is not implemented in this slice"
                            .to_string(),
                    ));
                }
            }
            Ok(())
        }
    }
}

fn parse_factor_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|e| QuantixError::DataParse(format!("invalid factor date `{}`: {}", value, e)))
}
