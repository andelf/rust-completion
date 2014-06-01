#![feature(managed_boxes, globs)]

extern crate debug;
extern crate syntax;
extern crate rustc;
extern crate collections;
extern crate rustdoc;

use rustc::{driver, middle};
//use rustc::driver::driver::CrateAnalysis;
use rustc::metadata::creader::Loader;
use rustc::middle::lint;

use syntax::parse::token;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::quote::rt::ToSource;
use std::cell::RefCell;
use std::os;
use collections::{HashSet, HashMap};
use rustdoc::core::{CrateAnalysis,DocContext, MaybeTyped, Typed, NotTyped};

use rustdoc::clean::Clean;
use std::local_data::Key;

//use rustdoc::visit_ast::RustdocVisitor;

use visitor::RustdocVisitor;

use utils::*;


pub static analysiskey: Key<CrateAnalysis> = &Key;
pub static ctxtkey: Key<@DocContext> = &Key;

mod utils;
mod visitor;

/// Parses, resolves, and typechecks the given crate
fn get_ast_and_resolve(cpath: &Path, libs: HashSet<Path>, cfgs: Vec<String>) -> (DocContext, CrateAnalysis) {
    use syntax::codemap::dummy_spanned;
    use rustc::driver::driver::{FileInput,
                                phase_1_parse_input,
                                phase_2_configure_and_expand,
                                phase_3_run_analysis_passes};
    use rustc::driver::config::build_configuration;

    let input = FileInput(cpath.clone());

    let sessopts = driver::config::Options {
        maybe_sysroot: Some(Path::new("/Users/wangshuyu/opt/rust/")),
        addl_lib_search_paths: RefCell::new(libs),
        crate_types: vec!(driver::config::CrateTypeDylib),
        lint_opts: vec!((lint::Warnings, lint::Allow)),
        ..rustc::driver::config::basic_options().clone()
    };

    let codemap = syntax::codemap::CodeMap::new();
    let diagnostic_handler = syntax::diagnostic::default_handler(syntax::diagnostic::Auto);
    let span_diagnostic_handler =
        syntax::diagnostic::mk_span_handler(diagnostic_handler, codemap);

    let sess = driver::session::build_session_(sessopts,
                                               Some(cpath.clone()),
                                               span_diagnostic_handler);

    let mut cfg = build_configuration(&sess);

    for cfg_ in cfgs.move_iter() {
        let cfg_ = token::intern_and_get_ident(cfg_.as_slice());
        cfg.push(@dummy_spanned(ast::MetaWord(cfg_)));
    }

    let krate = phase_1_parse_input(&sess, cfg, &input);
    let (krate, ast_map) = phase_2_configure_and_expand(&sess, &mut Loader::new(&sess),
                                                        krate, &from_str("rustdoc").unwrap());

    let driver::driver::CrateAnalysis {
        exported_items, public_items, ty_cx, ..
    } = phase_3_run_analysis_passes(sess, &krate, ast_map);

    (DocContext {
        krate: krate,
        maybe_typed: Typed(ty_cx),
        src: cpath.clone(),
        external_traits: RefCell::new(Some(HashMap::new())),
        external_typarams: RefCell::new(Some(HashMap::new())),
        external_paths: RefCell::new(Some(HashMap::new())),
        inlined: RefCell::new(Some(HashSet::new()))
    }, CrateAnalysis {
        exported_items: exported_items,
        public_items: public_items,
        external_traits: RefCell::new(None),
        external_typarams: RefCell::new(None),
        external_paths: RefCell::new(None),
        inlined: RefCell::new(None)
    })
}


pub fn run_core(libs: HashSet<Path>, cfgs: Vec<String>, path: &Path) {
    let (ctxt, analysis) = get_ast_and_resolve(path, libs, cfgs);

    let ctxt = @ctxt;
    ctxtkey.replace(Some(ctxt));

    let mut v = RustdocVisitor::new(ctxt, Some(&analysis));
    v.visit(&ctxt.krate);                                        // no clean here

    let module = v.module;
    println!("{}", "=".repeat(60));
    // println!("mudule => {:?}", module);
    for i in module.view_items.iter() {
        println!("vi => {}", view_item_to_str(i));
    }
    for i in module.structs.iter() {
        println!("struct name => {} id = {}", token::get_ident(i.name).get(), i.id)
    }

    for i in module.impls.iter() {
        println!("impl for => {:?}", i.for_);
    }
    for m in module.mods.iter() {
        println!("sub mod => {:?}", m.name.map(|ident| token::get_ident(ident).get().to_owned()));
    }
}


// ============================================================================



fn main() {
    let filename = os::args().get(1).to_owned();
    let cr = Path::new(filename);
    let libs = Vec::new();
    let cfgs = Vec::new();

    let cr = cr;
    let v = run_core(libs.move_iter().collect(),
             cfgs.move_iter().collect(),
             &cr);

    // let (krate, analysis) = std::task::try(proc() {
    //     let cr = cr;
    //     run_core(libs.move_iter().collect(),
    //              cfgs.move_iter().collect(),
    //              &cr)
    // }).ok().expect("parse input crate!");

    //println!("crate = > {:?}", krate);

    //let CrateAnalysis { ty_cx: ctx, public_items: pub_items, ..} = analysis;

    // for node_id in pub_items.iter() {

    //     print!("{} ", node_id);
    //     println!("-> {:?}", ctx.map.get(*node_id));
    // }
    println!("Hello World!");
}
