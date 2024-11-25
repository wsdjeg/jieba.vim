mod verified_case;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use verified_case::{
    NamedVerifiedCasesAndMod, VerifiedCases, VerifiedCasesHeader,
};

#[proc_macro_attribute]
pub fn verified_cases(attr: TokenStream, item: TokenStream) -> TokenStream {
    let header = parse_macro_input!(attr as VerifiedCasesHeader);
    let rest = parse_macro_input!(item as NamedVerifiedCasesAndMod);
    let verified_cases = VerifiedCases::new(header, rest);
    match verified_cases.verify_and_write_tests(false) {
        Err(message) => panic!("{}", message),
        Ok(out) => out.into(),
    }
}

#[proc_macro_attribute]
pub fn verified_cases_dry_run(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let header = parse_macro_input!(attr as VerifiedCasesHeader);
    let rest = parse_macro_input!(item as NamedVerifiedCasesAndMod);
    let verified_cases = VerifiedCases::new(header, rest);
    match verified_cases.verify_and_write_tests(true) {
        Err(message) => panic!("{}", message),
        Ok(out) => out.into(),
    }
}
