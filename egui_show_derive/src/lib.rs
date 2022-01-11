use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Data, DeriveInput, Field, Fields, Lit, LitStr, Meta, NestedMeta, Token, Type, Visibility,
};

struct RangeParser {
    from: Lit,
    _dot2: Token![..=],
    to: Lit,
}

impl Parse for RangeParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // panic!("{}", input.to_string());

        Ok(RangeParser {
            from: input.parse()?,
            _dot2: input.parse()?,
            to: input.parse()?,
        })
    }
}

impl ToTokens for RangeParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from = &self.from;
        let to = &self.to;
        tokens.extend(quote! {
            #from ..= #to
        })
    }
}

fn parse_attrs(field: &Field) -> Result<(bool, Option<LitStr>), TokenStream> {
    let mut skip = false;
    let mut range = None;

    for attr in field.attrs.iter() {
        let meta = attr
            .parse_meta()
            .map_err(|err| syn::Error::new(attr.span(), err).to_compile_error())?;

        match meta {
            Meta::List(l) => {
                for meta in l.nested.iter() {
                    match meta {
                        NestedMeta::Meta(meta) => match meta {
                            Meta::Path(path) => {
                                if path.segments.len() == 1
                                    && path.segments.first().unwrap().ident == "skip"
                                {
                                    skip = true;
                                } else {
                                    todo!()
                                }
                            }
                            Meta::NameValue(name_value) => {
                                if name_value.path.segments.len() == 1
                                    && name_value.path.segments.first().unwrap().ident == "range"
                                {
                                    match &name_value.lit {
                                        Lit::Str(s) => range = Some(s.clone()),
                                        _ => panic!("range has to be a str lit"),
                                    }
                                }
                            }
                            Meta::List(_) => todo!(),
                        },
                        NestedMeta::Lit(_) => todo!(),
                    }
                }
            }
            Meta::Path(_) => todo!(),
            Meta::NameValue(_) => todo!(),
        }
    }

    Ok((skip, range))
}

#[proc_macro_derive(EguiShow, attributes(egui_show))]
pub fn derive(tokens: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(tokens as DeriveInput);

    let ident = input.ident;

    let struct_data = match input.data {
        Data::Struct(s) => s,
        Data::Enum(_) => panic!("Enums not supported"),
        Data::Union(_) => panic!("Unions not supported"),
    };

    let fields = match struct_data.fields {
        Fields::Named(n) => n,
        Fields::Unnamed(_) => panic!("Tuple structs not supported"),
        Fields::Unit => panic!("Unit structs not supported"),
    }
    .named;

    // parse
    let fields: Result<Vec<(Field, bool, Option<LitStr>)>, TokenStream> = fields
        .into_iter()
        .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .map(|field| {
            let (skip, range) = parse_attrs(&field)?;
            Ok((field, skip, range))
        })
        .collect();

    let fields = match fields {
        Ok(ok) => ok,
        Err(err) => return err.into(),
    };

    let fields: Result<Vec<(Ident, Type, Option<RangeParser>)>, TokenStream> = fields
        .into_iter()
        .filter_map(|(field, skip, range)| {
            if skip {
                None
            } else {
                Some((field.ident?, field.ty, range))
            }
        })
        .map(|(ident, ty, range)| match range {
            Some(range) => syn::parse_str(range.value().as_str())
                .map(|s| (ident, ty, Some(s)))
                .map_err(|err| syn::Error::new(range.span(), err).to_compile_error()),
            None => Ok((ident, ty, None)),
        })
        .collect();

    let fields = match fields {
        Ok(ok) => ok,
        Err(err) => return err.into(),
    };

    // tokenize
    let fields = fields
        .into_iter()
        .fold(quote! {}, |acc, (ident, ty, range)| {
            if let Some(range) = range {
                quote! {
                    #acc
                    ui.add(egui::Label::new(stringify!(#ident)));
                    <#ty as egui_show::EguiShowValue>::show_range(&mut self.#ident, ui, #range);
                    ui.end_row();
                }
            } else {
                quote! {
                    #acc
                    ui.add(egui::Label::new(stringify!(#ident)));
                    <#ty as egui_show::EguiShowValue>::show(&mut self.#ident, ui);
                    ui.end_row();
                }
            }
        });

    (quote! {
        impl egui_show::EguiShow for #ident {
            fn show(&mut self, ui: &mut egui::Ui) {
                #fields
            }
        }
    })
    .into()
}
