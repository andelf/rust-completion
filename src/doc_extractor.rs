#![crate_id = "doc_extractor#0.1"]
#![crate_type = "dylib"]


extern crate rustdoc;

use rustdoc::clean;

use rustdoc::plugins::{PluginCallback, PluginResult, PluginJson};

// rustc src/doc_extractor.rs -o doc_extractor.dylib
// rustdoc -L. --plugin-path . --plugins dummy rust-sdl2/src/sdl2/lib.rs
// rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs

fn dump_item_enum(item: &clean::ItemEnum) {
    match *item {
        clean::ModuleItem(ref m) => {
            println!("|| items => {:?}", m.items);
        },
        _ => ()
    }
}

fn dump_item(item: &clean::Item) {
    println!("| item name => {:?}", item.name);
    println!("| vis => {:?}", item.visibility);
    for attr in item.attrs.iter() {
        println!("|   attr => {:?}", attr);
    }
    println!("| inner => {:?}", item.inner);
    dump_item_enum(&item.inner);
}

#[no_mangle]
pub fn rustdoc_plugin_entrypoint(c: clean::Crate) -> PluginResult {
    println!("loading extension ok!");
    println!("crate => {}", c.name);

    c.module.as_ref().map(dump_item);
    // externs is useless for our app
    // for ext in c.externs.iter() {
    //     println!("externs => {:?}", *ext);
    //     for attr in ext.ref1().attrs.iter() {
    //         println!("attrs => {:?}", attr);
    //     }
    // }

    (c, None)
}
