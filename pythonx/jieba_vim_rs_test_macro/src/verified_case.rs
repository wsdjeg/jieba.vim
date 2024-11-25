use jieba_vim_rs_test::verified_case::cases::{
    NmapECase, NmapWCase, OmapCWCase, OmapDWCase, OmapYWCase,
};
use jieba_vim_rs_test::verified_case::{
    verify_cases, Count, Mode, Motion, Operator,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprArray, ExprLit, ItemMod, Lit, Meta, Token};

/// The data for attribute `verified_case`.
pub struct VerifiedCase {
    buffer: Vec<String>,
    count: Count,
    d_special: bool,
}

struct NamedVerifiedCase {
    case: VerifiedCase,
    name: String,
}

fn parse_str_value(value: &Expr) -> Option<String> {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) => Some(lit_str.value()),
        _ => None,
    }
}

fn parse_str_array_value(value: &Expr) -> Option<Vec<String>> {
    match value {
        Expr::Array(ExprArray { elems, .. }) => Some(
            elems
                .iter()
                .filter_map(|el| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = el
                    {
                        Some(lit_str.value())
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        _ => None,
    }
}

fn parse_int_value<N>(value: &Expr) -> Option<N>
where
    N: FromStr,
    N::Err: fmt::Display,
{
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => Some(lit_int.base10_parse().unwrap()),
        _ => None,
    }
}

impl Parse for NamedVerifiedCase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut buffer: Option<Vec<String>> = None;
        let mut count: Option<u64> = None;
        let mut d_special = false;

        let pairs = input.parse_terminated(Meta::parse, Token![,])?;
        for pair in pairs {
            match pair {
                Meta::NameValue(name_value) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        match ident.to_string().as_str() {
                            "name" => {
                                name = Some(
                                    parse_str_value(&name_value.value).unwrap(),
                                )
                            }
                            "buffer" => {
                                buffer = Some(
                                    parse_str_array_value(&name_value.value)
                                        .unwrap(),
                                )
                            }
                            "count" => {
                                count = Some(
                                    parse_int_value(&name_value.value).unwrap(),
                                )
                            }
                            _ => (),
                        }
                    }
                }
                Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        match ident.to_string().as_str() {
                            "d_special" => d_special = true,
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(NamedVerifiedCase {
            name: name
                .ok_or(syn::Error::new(Span::call_site(), "Missing `name`"))?,
            case: VerifiedCase {
                buffer: buffer.ok_or(syn::Error::new(
                    Span::call_site(),
                    "Missing `buffer`",
                ))?,
                count: count.into(),
                d_special,
            },
        })
    }
}

pub struct NamedVerifiedCasesAndMod {
    cases: Vec<NamedVerifiedCase>,
    mod_name: String,
}

impl Parse for NamedVerifiedCasesAndMod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_mod: ItemMod = input.parse()?;
        let cases: Vec<_> = item_mod
            .attrs
            .iter()
            .filter_map(|a| {
                if a.path().is_ident("vcase") {
                    let case: NamedVerifiedCase = a.parse_args().unwrap();
                    Some(case)
                } else {
                    None
                }
            })
            .collect();
        Ok(NamedVerifiedCasesAndMod {
            cases,
            mod_name: item_mod.ident.to_string(),
        })
    }
}

/// The data for attribute `verified_cases` itself.
pub struct VerifiedCasesHeader {
    mode: Mode,
    operator: Operator,
    motion: Motion,
    timeout: u64,
    backend_path: String,
    buffer_type: String,
}

fn parse_str_value_into<T: FromStr>(
    value: &Expr,
    span: Span,
) -> Option<syn::Result<T>>
where
    T::Err: fmt::Display,
{
    let value = parse_str_value(value)?;
    Some(value.parse().map_err(|err| syn::Error::new(span, err)))
}

impl Parse for VerifiedCasesHeader {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut mode = None;
        let mut operator = None;
        let mut motion = None;
        let mut timeout = None;
        let mut backend_path = None;
        let mut buffer_type = None;

        let pairs = input.parse_terminated(Meta::parse, Token![,])?;
        for pair in pairs {
            match pair {
                Meta::NameValue(name_value) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        match ident.to_string().as_str() {
                            "mode" => {
                                let parsed: Mode = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                mode = Some(parsed);
                            }
                            "operator" => {
                                let parsed: Operator = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                operator = Some(parsed);
                            }
                            "motion" => {
                                let parsed: Motion = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                motion = Some(parsed);
                            }
                            "timeout" => {
                                let parsed =
                                    parse_int_value(&name_value.value).unwrap();
                                timeout = Some(parsed);
                            }
                            "backend_path" => {
                                let parsed =
                                    parse_str_value(&name_value.value).unwrap();
                                backend_path = Some(parsed);
                            }
                            "buffer_type" => {
                                let parsed =
                                    parse_str_value(&name_value.value).unwrap();
                                buffer_type = Some(parsed);
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }

        Ok(VerifiedCasesHeader {
            mode: mode
                .ok_or(syn::Error::new(Span::call_site(), "Missing `mode`"))?,
            operator: operator.unwrap_or(Operator::NoOp),
            motion: motion.ok_or(syn::Error::new(
                Span::call_site(),
                "Missing `motion`",
            ))?,
            timeout: timeout.ok_or(syn::Error::new(
                Span::call_site(),
                "Missing `timeout`",
            ))?,
            backend_path: backend_path.ok_or(syn::Error::new(
                Span::call_site(),
                "Missing `backend_path`",
            ))?,
            buffer_type: buffer_type.unwrap_or("Vec<String>".into()),
        })
    }
}

pub struct VerifiedCases {
    mode: Mode,
    operator: Operator,
    motion: Motion,
    timeout: u64,
    backend_path: syn::Path,
    buffer_type: syn::Type,
    group_name: String,
    cases: HashMap<String, Vec<VerifiedCase>>,
}

fn clone_cases_as<T, F>(
    cases: &HashMap<String, Vec<VerifiedCase>>,
    clone_func: F,
) -> HashMap<String, Vec<T>>
where
    F: Fn(&VerifiedCase) -> T,
{
    let mut new_map = HashMap::new();
    for (key, value) in cases.iter() {
        new_map
            .entry(key.clone())
            .or_insert_with(|| Vec::new())
            .extend(value.iter().map(|c| clone_func(c)));
    }
    new_map
}

impl VerifiedCases {
    pub fn new(
        header: VerifiedCasesHeader,
        flat_cases: NamedVerifiedCasesAndMod,
    ) -> Self {
        let mut cases = HashMap::new();
        for case in flat_cases.cases {
            cases
                .entry(case.name)
                .or_insert_with(|| Vec::new())
                .push(case.case);
        }
        Self {
            mode: header.mode,
            operator: header.operator,
            motion: header.motion,
            timeout: header.timeout,
            backend_path: syn::parse_str(&header.backend_path).unwrap(),
            buffer_type: syn::parse_str(&header.buffer_type).unwrap(),
            group_name: flat_cases.mod_name,
            cases,
        }
    }

    pub fn verify_and_write_tests(
        &self,
        skip_verify: bool,
    ) -> Result<TokenStream, String> {
        match (&self.mode, &self.operator, &self.motion) {
            (Mode::Normal, Operator::NoOp, Motion::W(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    NmapWCase::new(c.buffer.clone(), c.count, *word).unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_nmap_w_assertion(case_name, case_id, case, *word)
                }))
            }
            (Mode::Normal, Operator::NoOp, Motion::E(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    NmapECase::new(c.buffer.clone(), c.count, *word).unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_nmap_e_assertion(case_name, case_id, case, *word)
                }))
            }
            (Mode::Operator, Operator::Change, Motion::W(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapCWCase::new(c.buffer.clone(), c.count, *word).unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_c_w_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Delete, Motion::W(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapDWCase::new(c.buffer.clone(), c.count, *word).unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_d_w_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Yank, Motion::W(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapYWCase::new(c.buffer.clone(), c.count, *word).unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_y_w_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            _ => Err("Unsupported mode/operator/motion combination".into()),
        }
    }

    fn write_all_tests<T, F>(
        &self,
        cases: &HashMap<String, Vec<T>>,
        mut write_assertion_func: F,
    ) -> TokenStream
    where
        F: FnMut(&str, usize, &T) -> TokenStream,
    {
        let mut test_func_codes = Vec::new();
        for (case_name, sub_cases) in cases.iter() {
            for (i, case) in sub_cases.iter().enumerate() {
                let case_id = i + 1;
                test_func_codes
                    .push(write_assertion_func(case_name, case_id, case));
            }
        }
        let group_name: Ident = syn::parse_str(&self.group_name).unwrap();
        quote! {
            mod #group_name {
                #(#test_func_codes)*
            }
        }
    }

    fn write_nmap_w_assertion(
        &self,
        case_name: &str,
        case_id: usize,
        case: &NmapWCase,
        word: bool,
    ) -> TokenStream {
        let test_name: Ident =
            syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
        let backend_path = &self.backend_path;
        let buffer_type = &self.buffer_type;
        let timeout = self.timeout;

        let lnum_before = case.lnum_before;
        let lnum_after = case.lnum_after;
        let col_before = case.col_before;
        let col_after = case.col_after;
        let buffer = &case.buffer;
        let count = case.count.explicit();
        let case_desc = case.to_string();

        quote! {
            #[test]
            fn #test_name() {
                use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

                let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                let timing = AssertElapsed::tic(#timeout);
                let (lnum_after_pred, col_after_pred) = #backend_path.nmap_w(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                timing.toc();
                assert_eq!((lnum_after_pred, col_after_pred), (#lnum_after, #col_after), "\n{}", #case_desc);
            }
        }
    }

    fn write_nmap_e_assertion(
        &self,
        case_name: &str,
        case_id: usize,
        case: &NmapECase,
        word: bool,
    ) -> TokenStream {
        let test_name: Ident =
            syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
        let backend_path = &self.backend_path;
        let buffer_type = &self.buffer_type;
        let timeout = self.timeout;

        let lnum_before = case.lnum_before;
        let lnum_after = case.lnum_after;
        let col_before = case.col_before;
        let col_after = case.col_after;
        let buffer = &case.buffer;
        let count = case.count.explicit();
        let case_desc = case.to_string();

        quote! {
            #[test]
            fn #test_name() {
                use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

                let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                let timing = AssertElapsed::tic(#timeout);
                let (lnum_after_pred, col_after_pred) = #backend_path.nmap_e(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                timing.toc();
                assert_eq!((lnum_after_pred, col_after_pred), (#lnum_after, #col_after), "\n{}", #case_desc);
            }
        }
    }

    fn write_omap_c_w_assertion(
        &self,
        case_name: &str,
        case_id: usize,
        case: &OmapCWCase,
        word: bool,
    ) -> TokenStream {
        let test_name: Ident =
            syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
        let backend_path = &self.backend_path;
        let buffer_type = &self.buffer_type;
        let timeout = self.timeout;

        let lnum_before = case.lnum_before;
        let lnum_after = case.lnum_after;
        let col_before = case.col_before;
        let col_after = case.col_after;
        let buffer = &case.buffer;
        let count = case.count.explicit();
        let case_desc = case.to_string();

        quote! {
            #[test]
            fn #test_name() {
                use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

                let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                let timing = AssertElapsed::tic(#timeout);
                let (lnum_after_pred, col_after_pred) = #backend_path.omap_c_w(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                timing.toc();
                assert_eq!((lnum_after_pred, col_after_pred), (#lnum_after, #col_after), "\n{}", #case_desc);
            }
        }
    }

    fn write_omap_d_w_assertion(
        &self,
        case_name: &str,
        case_id: usize,
        case: &OmapDWCase,
        word: bool,
    ) -> TokenStream {
        let test_name: Ident =
            syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
        let backend_path = &self.backend_path;
        let buffer_type = &self.buffer_type;
        let timeout = self.timeout;

        let lnum_before = case.lnum_before;
        let lnum_after = case.lnum_after;
        let col_before = case.col_before;
        let col_after = case.col_after;
        let buffer = &case.buffer;
        let count = case.count.explicit();
        let case_desc = case.to_string();

        quote! {
            #[test]
            fn #test_name() {
                use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

                let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                let timing = AssertElapsed::tic(#timeout);
                let (lnum_after_pred, col_after_pred) = #backend_path.omap_w(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                timing.toc();
                assert_eq!((lnum_after_pred, col_after_pred), (#lnum_after, #col_after), "\n{}", #case_desc);
            }
        }
    }

    fn write_omap_y_w_assertion(
        &self,
        case_name: &str,
        case_id: usize,
        case: &OmapYWCase,
        word: bool,
    ) -> TokenStream {
        let test_name: Ident =
            syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
        let backend_path = &self.backend_path;
        let buffer_type = &self.buffer_type;
        let timeout = self.timeout;

        let lnum_before = case.lnum_before;
        let lnum_after = case.lnum_after;
        let col_before = case.col_before;
        let col_after = case.col_after;
        let buffer = &case.buffer;
        let count = case.count.explicit();
        let case_desc = case.to_string();

        quote! {
            #[test]
            fn #test_name() {
                use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

                let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                let timing = AssertElapsed::tic(#timeout);
                let (lnum_after_pred, col_after_pred) = #backend_path.omap_w(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                timing.toc();
                assert_eq!((lnum_after_pred, col_after_pred), (#lnum_after, #col_after), "\n{}", #case_desc);
            }
        }
    }
}
