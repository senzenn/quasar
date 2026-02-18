use syn::{FnArg, Item, Pat, Type};

/// Raw instruction data extracted from a `#[instruction(...)]` function.
pub struct RawInstruction {
    pub name: String,
    pub discriminator: Vec<u8>,
    pub accounts_type_name: String,
    pub args: Vec<(String, syn::Type)>,
}

/// Extract the program address from `declare_id!("...")`.
pub fn extract_program_id(file: &syn::File) -> Option<String> {
    for item in &file.items {
        if let Item::Macro(item_macro) = item {
            if item_macro.mac.path.is_ident("declare_id") {
                let lit: syn::LitStr = syn::parse2(item_macro.mac.tokens.clone()).ok()?;
                return Some(lit.value());
            }
        }
    }
    None
}

/// Extract the `#[program]` module name and its instruction functions.
pub fn extract_program_module(file: &syn::File) -> Option<(String, Vec<RawInstruction>)> {
    for item in &file.items {
        if let Item::Mod(item_mod) = item {
            if !has_program_attr(&item_mod.attrs) {
                continue;
            }

            let module_name = item_mod.ident.to_string();
            let mut instructions = Vec::new();

            if let Some((_, items)) = &item_mod.content {
                for item in items {
                    if let Item::Fn(func) = item {
                        if let Some(ix) = extract_instruction(func) {
                            instructions.push(ix);
                        }
                    }
                }
            }

            return Some((module_name, instructions));
        }
    }
    None
}

fn has_program_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("program"))
}

fn extract_instruction(func: &syn::ItemFn) -> Option<RawInstruction> {
    // Find the #[instruction(discriminator = ...)] attribute
    let attr = func
        .attrs
        .iter()
        .find(|a| a.path().is_ident("instruction"))?;

    let discriminator = parse_discriminator_attr(attr)?;
    let name = func.sig.ident.to_string();
    let accounts_type_name = extract_ctx_type_name(&func.sig)?;

    // Extract extra args (everything after the first `ctx` parameter)
    let args: Vec<(String, syn::Type)> = func
        .sig
        .inputs
        .iter()
        .skip(1)
        .filter_map(|arg| match arg {
            FnArg::Typed(pt) => {
                let name = match &*pt.pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => return None,
                };
                Some((name, (*pt.ty).clone()))
            }
            _ => None,
        })
        .collect();

    Some(RawInstruction {
        name,
        discriminator,
        accounts_type_name,
        args,
    })
}

/// Parse `#[instruction(discriminator = N)]` or `#[instruction(discriminator = [N, M])]`.
fn parse_discriminator_attr(attr: &syn::Attribute) -> Option<Vec<u8>> {
    let tokens = attr.meta.require_list().ok()?.tokens.clone();
    let tokens_str = tokens.to_string();

    // Find "discriminator = ..." pattern
    let eq_pos = tokens_str.find('=')?;
    let value_str = tokens_str[eq_pos + 1..].trim();

    if value_str.starts_with('[') {
        // Array form: [0, 1, 2]
        let inner = value_str.trim_start_matches('[').trim_end_matches(']');
        let bytes: Vec<u8> = inner
            .split(',')
            .filter_map(|s| s.trim().parse::<u8>().ok())
            .collect();
        if bytes.is_empty() {
            None
        } else {
            Some(bytes)
        }
    } else {
        // Single value form: 0
        let byte: u8 = value_str
            .trim_end_matches(|c: char| !c.is_ascii_digit())
            .parse()
            .ok()?;
        Some(vec![byte])
    }
}

/// Extract the inner type name `T` from `Ctx<T>` in the first parameter.
fn extract_ctx_type_name(sig: &syn::Signature) -> Option<String> {
    let first = sig.inputs.first()?;
    let typed = match first {
        FnArg::Typed(pt) => pt,
        _ => return None,
    };

    match &*typed.ty {
        Type::Path(type_path) => {
            let last_seg = type_path.path.segments.last()?;
            match &last_seg.arguments {
                syn::PathArguments::AngleBracketed(args) => match args.args.first()? {
                    syn::GenericArgument::Type(Type::Path(inner_path)) => {
                        Some(inner_path.path.segments.last()?.ident.to_string())
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}
