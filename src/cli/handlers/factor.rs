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
        FactorCommands::Compute { .. } => Err(QuantixError::Unsupported(
            "factor compute requires a data loader; P1 CLI data-loader wiring is implemented in the next step"
                .to_string(),
        )),
    }
}
