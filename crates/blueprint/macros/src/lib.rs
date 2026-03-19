use quote::quote;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    Ident, LitInt, LitStr, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};
use toml::Table;

struct Blueprint {
    path: PathBuf,
    ty: Type,
    layouts: Vec<Layout>,
}

impl Parse for Blueprint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit_path: LitStr = input.parse()?;
        let path = lit_path.value().into();
        _ = input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        _ = input.parse::<Token![,]>()?;
        let layouts = parse_zero_or_more(input);
        Ok(Self { path, ty, layouts })
    }
}

fn parse_zero_or_more<T: Parse>(input: ParseStream) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

struct Layout {
    name: Ident,
    num_bits: u8,
}

impl Parse for Layout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        _ = input.parse::<Token![:]>()?;
        let lit_num_bits: LitInt = input.parse()?;
        let num_bits = lit_num_bits.base10_parse()?;
        _ = input.parse::<Token![,]>()?;
        Ok(Self { name, num_bits })
    }
}

fn make_entry(blueprint: &Blueprint) -> proc_macro2::TokenStream {
    let mut current_shift = 0u32;
    let mut field_info = Vec::new();

    for layout in &blueprint.layouts {
        let mask = (1u128 << layout.num_bits) - 1;
        field_info.push((&layout.name, current_shift, mask));
        current_shift += layout.num_bits as u32;
    }

    let ty = &blueprint.ty;

    let getters = field_info.iter().map(|(name, shift, mask)| {
        quote! {
            pub const fn #name(&self) -> #ty {
                ((self.0 >> #shift) & (#mask as #ty))
            }
        }
    });

    let constructor_args = field_info.iter().map(|(name, _, _)| quote!(#name: #ty));
    let assignments = field_info.iter().map(|(name, shift, mask)| {
        quote! { data |= (((#name as #ty) & (#mask as #ty)) << #shift); }
    });

    quote! {
        pub struct Entry(pub #ty);

        impl Default for Entry {
            fn default() -> Self {
                Self::MISSING
            }
        }

        impl Entry {
            pub const MISSING: Self = Self(#ty::MIN);

            pub const fn new(#(#constructor_args),*) -> Self {
                let mut data: #ty = 0;
                #(#assignments)*
                Self(data)
            }

            #(#getters)*
        }
    }
}

fn make_toml_dictionary(blueprint: &Blueprint) -> proc_macro2::TokenStream {
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let full_path = Path::new(&root).join(&blueprint.path);
    let content = fs::read_to_string(&full_path)
        .expect(&format!("Could not find TOML file at {:?}", full_path));

    let toml_data: Table = toml::from_str(&content).expect("Invalid TOML");
    let ty = &blueprint.ty;

    let entries = toml_data
        .get("entries")
        .and_then(|v| v.as_array())
        .expect("TOML must have a [[entries]] list")
        .iter()
        .map(|e| {
            let table = e.as_table().expect("Entry is not a table");

            let vals = blueprint.layouts.iter().map(|layout| {
                let key = layout.name.to_string();
                let val = table
                    .get(&key)
                    .and_then(|v| v.as_integer())
                    .expect(&format!("Missing integer field '{}' in TOML entry", key));

                quote! { (#val as #ty) }
            });

            quote! { Entry::new(#(#vals),*) }
        });

    quote! {
        pub const ENTRIES: &[Entry] = &[
            #(#entries),*
        ];
    }
}

/// Bakes a TOML file into a bit-packed static array of entries at compile time.
///
/// This macro generates a bit-packed `Entry` struct and a static `ENTRIES` slice
/// containing data parsed from the provided TOML file.
///
/// # Arguments
/// * `path`: A string literal path to the TOML file (relative to the project root).
/// * `type`: The underlying integer type to use for storage (e.g., `u32`, `u64`).
/// * `fields`: A list of `name: bits` pairs defining how the integer is partitioned.
///
/// # The Generated `Entry`
/// The macro generates a `struct Entry(pub $type)`. For every field defined in the
/// macro input, it generates a `const fn <field_name>(&self)` that extracts
/// those specific bits.
///
/// # The `ENTRIES` Array
/// The macro reads the TOML file at the provided path. It expects an `[[entries]]`
/// array where each entry contains keys matching the field names defined in the macro.
/// It then populates a `pub const ENTRIES: &[Entry]` with this data.
///
/// # Examples
///
/// ```rust
/// bake_toml! {
///     "data/config.toml",
///     u32,
///     id: 8,      // 8 bits for ID
///     value: 16,  // 16 bits for Value
///     flag: 1,    // 1 bit for a flag
/// }
///
/// fn main() {
///     let first = &ENTRIES[0];
///     println!("ID: {}, Value: {}", first.id(), first.value());
/// }
/// ```
///
/// **Expected TOML structure:**
/// ```toml
/// [[entries]]
/// id = 1
/// value = 500
/// flag = 1
///
/// [[entries]]
/// id = 2
/// value = 1000
/// flag = 0
/// ```
#[proc_macro]
pub fn bake_toml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let blueprint = parse_macro_input!(input as Blueprint);
    let entry_logic = make_entry(&blueprint);
    let toml_dictionary = make_toml_dictionary(&blueprint);

    quote! {
        #entry_logic
        #toml_dictionary
    }
    .into()
}
