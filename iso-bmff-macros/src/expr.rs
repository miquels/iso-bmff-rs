use proc_macro2::{TokenStream, Span};
use syn::{Ident, Result, Error};
use syn::{parenthesized, token, token::Token};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use quote::ToTokens;

macro_rules! def_binop {
    ($($op:ident),+) => {
        pub enum BinOp {
            $(
                $op(token::$op),
            )+
        }

        impl BinOp {
            pub fn peek(input: ParseStream) -> bool {
                $(
                    input.peek(token::$op)
                )||+
            }
            pub fn parse(input: ParseStream) -> Result<Self> {
                $(
                    if input.peek(token::$op) {
                        let t = input.parse::<token::$op>()?;
                        return Ok(BinOp::$op(t));
                    }
                )+
                Err(input.error("expected valid operator"))
            }
        }
        impl ToTokens for BinOp {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                match self {
                    $( &BinOp::$op(ref inner) => inner.to_tokens(tokens), )*
                }
            }
        }
        impl std::fmt::Debug for BinOp {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $( &BinOp::$op(..) => write!(f, "{}", token::$op::display()), )*
                }
            }
        }
    }
}

def_binop!{Add, Sub, Star, Div, And, OrOr, Le, Lt, Ge, Gt, EqEq, Ne, Shl, Shr}

// Expression.
pub enum Expr {
    // "foo"
    LitStr(syn::LitStr),
    // 42
    LitInt(syn::LitInt),
    // expr between parentheses: (a + b)
    Paren{
        paren_token:    token::Paren,
        expr:   Box<Expr>,
    },
    // variable
    Variable(Ident),
    // a + b
    Binary {
        left:   Box<Expr>,
        op:     BinOp,
        right:  Box<Expr>,
    },
}

impl Expr {
    pub fn new_lit_int(val: impl ToString, span: Span) -> Expr {
        Expr::LitInt(syn::LitInt::new(&val.to_string(), span))
    }

    #[allow(dead_code)]
    pub fn get_lit_int(&self) -> Result<u32> {
        if let &Expr::LitInt(ref lit_int) = self {
            return lit_int.base10_parse::<u32>();
        }
        Err(Error::new(self.span(), "expected number"))
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &Expr::LitInt(ref t) => write!(f, "Expr::LitInt({})", t.to_string()),
            &Expr::LitStr(ref t) => write!(f, "Expr::LitStr({})", t.value()),
            &Expr::Paren{ref expr, ..} => f.debug_tuple("Expr::Paren").field(expr).finish(),
            &Expr::Variable(ref t) => write!(f, "Expr::Variable({})", t.to_string()),
            &Expr::Binary{ref left, ref op, ref right} => {
                f.debug_struct("Expr::Binary")
                    .field("left", left)
                    .field("op", op)
                    .field("right", right)
                    .finish()
            }
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            &Expr::LitStr(ref t) => t.to_tokens(tokens),
            &Expr::LitInt(ref t) => t.to_tokens(tokens),
            &Expr::Variable(ref t) => t.to_tokens(tokens),
            &Expr::Paren{ref paren_token, ref expr} => {
                paren_token.surround(tokens, |tokens| {
                    expr.to_tokens(tokens);
                })
            },
            &Expr::Binary{ref left, ref op, ref right} => {
                left.to_tokens(tokens);
                op.to_tokens(tokens);
                right.to_tokens(tokens);
            }
        }
    }
}


impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        // First check for some kind of value.
        let lookahead = input.lookahead1();
        let left = if lookahead.peek(token::Paren) {
            let content;
            let paren_token = parenthesized!(content in input);
            let expr = Box::new(content.parse()?);
            Expr::Paren{ paren_token, expr }
        } else if lookahead.peek(syn::LitStr) {
            Expr::LitStr(input.parse()?)
        } else if lookahead.peek(syn::LitInt) {
            Expr::LitInt(input.parse()?)
        } else if lookahead.peek(Ident::peek_any) {
            Expr::Variable(input.call(Ident::parse_any)?)
        } else {
            return Err(lookahead.error());
        };
        // not followed by an operator? End of expression.
        if !BinOp::peek(&input) {
            return Ok(left);
        }
        // Operator, and an expression on the right hand side.
        let op = BinOp::parse(&input)?;
        let right: Expr = input.parse()?;
        Ok(Expr::Binary{
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }
}

