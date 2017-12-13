#![feature(proc_macro)]

#[macro_use]
extern crate quote;

extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::FnArg::{Ignored, Captured};
use syn::{Ident, Ty, Pat, Path};
use quote::{Tokens, ToTokens};

#[proc_macro_attribute]
pub fn get(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::GET").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn post(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::POST").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn patch(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::PATCH").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn put(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::PUT").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn delete(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::DELETE").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn options(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::OPTIONS").unwrap(), opts, item)
}

#[proc_macro_attribute]
pub fn head(opts: TokenStream, item: TokenStream) -> TokenStream {
    impl_route_rewrite(syn::parse_expr("::shio::http::Method::HEAD").unwrap(), opts, item)
}

fn impl_route_rewrite(meth: syn::Expr, opts: TokenStream, item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let item = syn::parse_item(&item).expect("unable to parse item associated to get attribute");

    match item.node {
        syn::ItemKind::Fn(_, _, _, _, _, _) => {}
        _ => panic!("get attribute is only for functions"),
    }

    let opts = opts.to_string();
    let opts = syn::parse_token_trees(&opts).expect("unable to parse options of get attribute");
    let opts = &opts[0];

    let tts = match *opts {
        syn::TokenTree::Delimited(ref delim) => &delim.tts,
        _ => panic!("unvalid attribute options"),
    };
    let tt1 = &tts[0];
    let tok = match *tt1 {
        syn::TokenTree::Token(ref tok) => tok,
        _ => panic!("expected a token as first attribute option"),
    };
    let lit = match *tok {
        syn::Token::Literal(ref lit) => lit,
        _ => panic!("expected a literal as first attribute option"),
    };
    match *lit {
        syn::Lit::Str(_, _) => {}
        _ => panic!("expected a string literal as first attribute option"),
    };

    Route {
        handler: item,
        shio_method: meth,
        route: lit.clone(),
    }.create_new_token_stream()
}

struct Route {
    handler: syn::Item,
    shio_method: syn::Expr,
    route: syn::Lit,
}

impl Route {
    fn create_new_token_stream(mut self) -> TokenStream {
        let new_ident = syn::Ident::from(format!("__shio_handler_{}", self.handler.ident));
        let convert_ident = syn::Ident::from(format!("__shio_convert_{}", self.handler.ident));
        let prev_ident = self.handler.ident.clone();
        self.handler.ident = new_ident.clone();

        let convert_function = self.create_convert_function(&new_ident);

        let Route { handler, shio_method, route } = self;

        let tokens = quote! {
            #handler
            #[allow(non_camel_case_types)]
            pub struct #prev_ident;
            fn #convert_ident(ctx: Context) -> Response {
                #convert_function
            }
            impl Into<::shio::router::Route> for #prev_ident {
                fn into(self) -> ::shio::router::Route {
                    (#shio_method, #route, #convert_ident).into()
                }
            }
        };

        tokens.parse().unwrap()
    }

    fn create_convert_function(&mut self, func_name: &syn::Ident) -> quote::Tokens {
        if let syn::ItemKind::Fn(ref func_def, _, _, _, _, _) = self.handler.node {
            let mut result = quote::Tokens::new();
            let mut current = 0;
            let mut params = Vec::new();
            for func_parameter in &func_def.inputs {
                match func_parameter {
                    &Captured(ref pat, ref ty) => {
                        let param_name = match pat {
                            &Pat::Ident(_, ref ident, _) => Some(ident.as_ref()),
                            &Pat::Wild => None,
                            _ => panic!("Unexpected captured parameter name {:?}", pat)
                        };
                        let param_type = match ty {
                            &Ty::Path(_, ref path) => quote::Ident::from(path.segments[0].ident.as_ref()),
                            _ => panic!("Unexpected captured parameter type {:?}", ty)
                        };
                        let mv = MultiVariable { index: current };
                        result.append(quote!{
                            let #mv = ctx.acquire_by_name::<#param_type> ( #param_name );
                        });

                        params.push(mv);
                        current+=1;
                    }
                    &Ignored(ref ty) => {
                        let mv = MultiVariable { index: current };
                        params.push(mv);
                        current+=1;
                    }
                    _ => {}
                }
            }
            result.append(quote!{ 
                #func_name(#(#params),*)
            });
            result
        } else {
            panic!("Never happen!");
        }
    }
}

struct MultiVariable {
    index: i32,
}

impl ToTokens for MultiVariable {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let mut token = String::with_capacity(10);
        token.push_str("param");
        token.push_str(self.index.to_string().as_ref());
        tokens.append(token);
    }
}
