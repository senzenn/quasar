pub mod accounts;
pub mod errors;
pub mod helpers;
pub mod module_resolver;
pub mod program;
pub mod state;

use std::path::Path;

use crate::types::*;

/// All data extracted from parsing a quasar program crate.
pub struct ParsedProgram {
    pub program_id: String,
    pub program_name: String,
    pub version: String,
    pub instructions: Vec<program::RawInstruction>,
    pub accounts_structs: Vec<accounts::RawAccountsStruct>,
    pub state_accounts: Vec<state::RawStateAccount>,
    pub errors: Vec<IdlError>,
}

/// Parse an entire quasar program crate and produce a `ParsedProgram`.
pub fn parse_program(crate_root: &Path) -> ParsedProgram {
    // 1. Resolve all source files
    let files = module_resolver::resolve_crate(crate_root);

    // 2. Find lib.rs (first resolved file that has declare_id! or #[program])
    let lib_file = files
        .iter()
        .find(|f| f.path.ends_with("lib.rs"))
        .expect("could not find lib.rs");

    // 3. Extract program ID
    let program_id = program::extract_program_id(&lib_file.file)
        .expect("could not find declare_id! in lib.rs");

    // 4. Extract program module and instructions
    let (program_name, instructions) = program::extract_program_module(&lib_file.file)
        .expect("could not find #[program] module in lib.rs");

    // 5. Collect all #[derive(Accounts)] structs across all files
    let mut accounts_structs = Vec::new();
    for file in &files {
        accounts_structs.extend(accounts::extract_accounts_structs(&file.file));
    }

    // 6. Collect all #[account(discriminator = N)] state structs
    let mut state_accounts = Vec::new();
    for file in &files {
        state_accounts.extend(state::extract_state_accounts(&file.file));
    }

    // 7. Collect all #[error_code] enums
    let mut all_errors = Vec::new();
    for file in &files {
        all_errors.extend(errors::extract_errors(&file.file));
    }

    // 8. Read version from Cargo.toml
    let version = read_cargo_version(crate_root).unwrap_or_else(|| "0.1.0".to_string());

    ParsedProgram {
        program_id,
        program_name,
        version,
        instructions,
        accounts_structs,
        state_accounts,
        errors: all_errors,
    }
}

/// Build the final `Idl` from parsed program data.
pub fn build_idl(parsed: ParsedProgram) -> Idl {
    let instructions: Vec<IdlInstruction> = parsed
        .instructions
        .iter()
        .map(|ix| {
            // Look up the accounts struct by name
            let accounts_items = parsed
                .accounts_structs
                .iter()
                .find(|s| s.name == ix.accounts_type_name)
                .map(|s| accounts::to_idl_accounts(s))
                .unwrap_or_default();

            let args: Vec<IdlField> = ix
                .args
                .iter()
                .map(|(name, ty)| {
                    let type_name = helpers::simple_type_name(ty);
                    IdlField {
                        name: helpers::to_camel_case(name),
                        ty: helpers::map_type(&type_name),
                    }
                })
                .collect();

            IdlInstruction {
                name: helpers::to_camel_case(&ix.name),
                discriminator: ix.discriminator.clone(),
                accounts: accounts_items,
                args,
            }
        })
        .collect();

    let account_defs: Vec<IdlAccountDef> = parsed
        .state_accounts
        .iter()
        .map(state::to_idl_account_def)
        .collect();

    let type_defs: Vec<IdlTypeDef> = parsed
        .state_accounts
        .iter()
        .map(state::to_idl_type_def)
        .collect();

    Idl {
        address: parsed.program_id,
        metadata: IdlMetadata {
            name: parsed.program_name,
            version: parsed.version,
            spec: "0.1.0".to_string(),
        },
        instructions,
        accounts: account_defs,
        types: type_defs,
        errors: parsed.errors,
    }
}

fn read_cargo_version(crate_root: &Path) -> Option<String> {
    let cargo_path = crate_root.join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    let package = table.get("package")?.as_table()?;
    package
        .get("version")?
        .as_str()
        .map(|s| s.to_string())
}
