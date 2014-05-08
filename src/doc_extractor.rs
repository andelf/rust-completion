#![crate_id = "doc_extractor#0.1"]
#![crate_type = "dylib"]


extern crate rustdoc;
extern crate syntax;


use syntax::ast;
use rustdoc::clean;

use rustdoc::plugins::{PluginCallback, PluginResult, PluginJson};

// rustc src/doc_extractor.rs -o doc_extractor.dylib
// rustdoc -L. --plugin-path . --plugins dummy rust-sdl2/src/sdl2/lib.rs
// rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs

pub trait Extractable {
    fn extract(&self);
}


fn dump_item_enum(item: &clean::ItemEnum, ident_level: uint) {
    match *item {
        clean::ModuleItem(ref m) => {
            println!("{}!A Module", " ".repeat(ident_level));
            println!("{}| module items => {:?}", " ".repeat(ident_level), m.items);
            for item in m.items.iter() {
                dump_item(item, ident_level + 2)
            }
        },
        clean::ViewItemItem(ref v) => {
            println!("{}!A ViewItem", " ".repeat(ident_level));
            match v.inner {
                clean::Import(ref vpath) => {
                    print!("{}| ", " ".repeat(ident_level));
                    println!("{:?}", vpath);
                }
                _ => {
                    print!("{}| ", " ".repeat(ident_level));
                    println!("{}", "extern create");
                }
            }
        }
        ref i => {
            println!("{}!{:?}", " ".repeat(ident_level), i)
        }
    }
}

fn dump_item(item: &clean::Item, ident_level: uint) {
    println!("{}| item name => {:?}", " ".repeat(ident_level), item.name);
    println!("{}| vis => {:?}", " ".repeat(ident_level), item.visibility);

    // for attr in item.attrs.iter() {
    //     println!("{}| attr => {:?}", " ".repeat(ident_level), attr);
    // }
    //println!("{}| inner => {:?}", " ".repeat(ident_level), item.inner);
    if item.visibility.expect("visibility here") == ast::Public {
        dump_item_enum(&item.inner, ident_level);
    }
}

#[no_mangle]
pub fn rustdoc_plugin_entrypoint(c: clean::Crate) -> PluginResult {
    println!("loading extension ok!");
    println!("crate => {}", c.name);

    c.module.as_ref().map(|m| dump_item(m, 0));
    // externs is useless for our app
    // for ext in c.externs.iter() {
    //     println!("externs => {:?}", *ext);
    //     for attr in ext.ref1().attrs.iter() {
    //         println!("attrs => {:?}", attr);
    //     }
    // }

    (c, None)
}
