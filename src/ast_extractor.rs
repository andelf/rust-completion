#![feature(managed_boxes)]

extern crate syntax;
extern crate rustc;
extern crate collections;

use rustc::{driver, middle};
use rustc::driver::driver::CrateAnalysis;
use rustc::metadata::creader::Loader;
use rustc::middle::privacy;
use rustc::middle::lint;
use syntax::parse;
use syntax::parse::token;
use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::quote::rt::ToSource;
use std::cell::RefCell;
use std::os;
use collections::HashSet;

pub enum MaybeTyped {
    Typed(middle::ty::ctxt),
    NotTyped(driver::session::Session)
}


pub struct DocContext {
    pub krate: ast::Crate,
    pub src: Path,
}

// copied from rustdoc
/// Parses, resolves, and typechecks the given crate
fn get_ast_and_resolve(cpath: &Path, libs: HashSet<Path>, cfgs: Vec<~str>) -> (ast::Crate, CrateAnalysis) {
    use syntax::codemap::dummy_spanned;
    use rustc::driver::driver::{FileInput,
                                phase_1_parse_input,
                                phase_2_configure_and_expand,
                                phase_3_run_analysis_passes};
    use rustc::driver::config::build_configuration;

    let input = FileInput(cpath.clone());

    let sessopts = rustc::driver::config::Options {
        // rust sys rootos::self_exe_path().unwrap().dir_path()),
        maybe_sysroot: Some(Path::new("/Users/wangshuyu/opt/rust/")),
        addl_lib_search_paths: RefCell::new(libs),
        crate_types: vec!(driver::config::CrateTypeDylib),
        lint_opts: vec!((lint::Warnings, lint::allow)),
        ..rustc::driver::config::basic_options().clone()
    };

    let sess = driver::session::build_session(sessopts, Some(cpath.clone()));

    let mut cfg = build_configuration(&sess);
    for cfg_ in cfgs.move_iter() {
        let cfg_ = token::intern_and_get_ident(cfg_);
        cfg.push(@dummy_spanned(ast::MetaWord(cfg_)));
    }

    // input => AST
    let krate = phase_1_parse_input(&sess, cfg, &input);


    let (krate, ast_map) = phase_2_configure_and_expand(&sess, &mut Loader::new(&sess),
                                                        krate, &from_str("rustdoc").unwrap());

    // pub public_items: PublicItems,
    let anlys : CrateAnalysis = phase_3_run_analysis_passes(sess, &krate, ast_map);
    (krate, anlys)
}


pub fn run_core(libs: HashSet<Path>, cfgs: Vec<~str>, path: &Path) -> (ast::Crate, CrateAnalysis) {
    let (krate, analysis) = get_ast_and_resolve(path, libs, cfgs);
    (krate, analysis)
}


// ============================================================================

pub struct AstVisitor<'a> {
    krate: &'a ast::Crate,
    analysis: &'a CrateAnalysis,
}

impl<'a> AstVisitor<'a> {
    pub fn new<'b>(krate: &'b ast::Crate, analysis: &'b CrateAnalysis) -> AstVisitor<'b> {
        AstVisitor{ krate: krate, analysis: analysis }
    }

    pub fn visit(&mut self) {
        let krate = self.krate;
        self.visit_mod_contents(krate.span, krate.attrs.iter().map(|&x| x).collect(),
                                ast::Public, ast::CRATE_NODE_ID, &krate.module, None);
    }

    pub fn visit_mod_contents(&self, span: Span, attrs: Vec<ast::Attribute> ,
                              vis: ast::Visibility, id: ast::NodeId,
                              m: &ast::Mod,
                              name: Option<ast::Ident>) -> () {
        for i in m.items.iter() {
            // visit item
            println!("item => {:?}", i);
            println!("{}", item_to_str(*i));

        }

    }

}

fn main() {
    let filename = os::args().get(1).to_owned();
    let cr = Path::new(filename);
    let libs = Vec::new();
    let cfgs = Vec::new();
    let (krate, analysis) = {
        let cr = cr;
        run_core(libs.move_iter().collect(),
                 cfgs.move_iter().collect(),
                 &cr)
    };
    // let (krate, analysis) = std::task::try(proc() {
    //     let cr = cr;
    //     run_core(libs.move_iter().collect(),
    //              cfgs.move_iter().collect(),
    //              &cr)
    // }).ok().expect("parse input crate!");

    println!("crate = > {:?}", krate);
    println!(": {}", rustc::driver::driver::host_triple());

    //let CrateAnalysis { ty_cx: ctx, public_items: pub_items, ..} = analysis;
    let mut visitor = AstVisitor::new(&krate, &analysis);
    visitor.visit();

    // for node_id in pub_items.iter() {

    //     print!("{} ", node_id);
    //     println!("-> {:?}", ctx.map.get(*node_id));
    // }
    println!("Hello World!");
}

fn view_item_to_str(i: &ast::ViewItem) {
    let mut ret = StrBuf::new();
    if i.vis == ast::Public {
        ret.push_str("pub")
    }
    match i.node {
        ast::ViewItemExternCrate(..) => {
            println!("extern crate");
        }
        ast::ViewItemUse(..) => {
            println!("use");
        }
    }
}

fn item_to_str(i: &ast::Item) -> ~str {
    let mut ret = StrBuf::new();
    if i.vis == ast::Public {
        ret.push_str(format!("pub "));
    }
    match i.node {
        ast::ItemStatic(..) => {
            ret.push_str(format!("ItemStatic"));
        }
        ast::ItemFn(..) => {
            ret.push_str(format!("ItemFn"));
        }
        ast::ItemMod(..) => {
            ret.push_str(format!("ItemMod"));
        }
        ast::ItemForeignMod(..) => {
            ret.push_str(format!("ItemForeignMod"));
        }
        ast::ItemTy(..) => {
            ret.push_str(format!("ItemTy"));
        }
        ast::ItemEnum(..) => {
            ret.push_str(format!("ItemEnum"));
        }
        ast::ItemStruct(..) => {
            ret.push_str(format!("ItemStruct"));
        }
        ast::ItemTrait(..) => {
            ret.push_str(format!("ItemTrait"));
        }
        ast::ItemImpl(..) => {
            ret.push_str(format!("ItemImpl"));
        }
        ast::ItemMac(..) => {
            ret.push_str(format!("ItemMac"));
        }
    }
    ret.push_str(format!("ITEM: {} ", i.ident.to_source()));
    ret.to_owned()
}
