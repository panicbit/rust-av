extern crate bindgen;
extern crate syn;
extern crate quote;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use syn::{Item, ItemKind, Visibility, ConstExpr, Expr, ExprKind};
use quote::{Tokens, ToTokens};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let prefix = env::var("RUST_FFMPEG_PREFIX").unwrap_or("/usr".to_owned());
    let out_path = Path::new(&out_dir).join("ffi.rs");
    let mut out_file = File::create(out_path).unwrap();

    let bindings = bindgen::builder()
        .header("ffi.h")
        .clang_arg(format!("-I{}/include", prefix))
        .no_unstable_rust()
        .whitelisted_type("AV.*")
        .whitelisted_var("AV.*")
        .whitelisted_var("FF.*")
        .whitelisted_function("av.*")
        .whitelisted_type("SWS.*")
        .whitelisted_var("SWS.*")
        .whitelisted_function("sws.*")
        .whitelisted_type("RUST_AV.*")
        .whitelisted_var("SEEK_.*")
        .whitelisted_type(".*_t")
        .generate()
        .unwrap()
        .to_string();

    let mut krate = syn::parse_crate(&bindings).unwrap();
    let const_enum_index = krate.items.iter().position(|item| item.ident == "RUST_AV_CONSTANTS").expect("RUST_AV_CONSTANTS not found");
    let const_enum = krate.items.remove(const_enum_index);
    let variants = match const_enum.node {
        ItemKind::Enum(variants, _) => variants,
        _ => panic!("RUST_AV_CONSTANTS is not an enum"),
    };

    for variant in variants {
        let variant_ident: Vec<&str> = variant.ident.as_ref().split("__").collect();
        let ty_prefix = match variant_ident[0] {
            "RUST" => "",
            "RUST_OS_RAW" => "::std::os::raw::",
            _ => panic!("Unknown type prefix"),
        };
        let ty = syn::parse_type(&format!("{}{}", ty_prefix, variant_ident[1])).unwrap();
        let ident = syn::parse_ident(variant_ident[2]).unwrap();
        let expr = const_expr_into_expr(variant.discriminant.expect("Discriminant missing from RUST_AV_CONSTANTS variant"));

        let item = Item {
            ident: ident.into(),
            vis: Visibility::Public,
            attrs: vec![],
            node: ItemKind::Const(Box::new(ty), expr),
        };

        krate.items.push(item);
    }

    let mut tokens = Tokens::new();
    krate.to_tokens(&mut tokens);
    write!(out_file, "{}", tokens).unwrap();


    println!("cargo:rustc-link-search=native={}/lib", prefix);
    println!("cargo:rustc-link-lib=dylib=avutil");
    println!("cargo:rustc-link-lib=dylib=avformat");
    println!("cargo:rustc-link-lib=dylib=avcodec");
    println!("cargo:rustc-link-lib=dylib=avdevice");
    println!("cargo:rustc-link-lib=dylib=avfilter");
    println!("cargo:rustc-link-lib=dylib=swresample");
    println!("cargo:rustc-link-lib=dylib=swscale");
}

fn const_expr_into_expr(const_expr: ConstExpr) -> Box<Expr> {
    let node = match const_expr {
        ConstExpr::Lit(lit) => ExprKind::Lit(lit),
        ConstExpr::Unary(op, expr) => ExprKind::Unary(op, const_expr_into_expr(*expr)),
        expr => panic!("Unexpected discriminant kind: {:?}", expr),
    };

    Box::new(Expr {
        node: node,
        attrs: vec![],
    })
}
