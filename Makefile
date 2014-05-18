RUSTDOC ?= rustdoc
RUSTC ?= rustc
RUST_SRC_DIR ?= ~/Repos/rust

.PHONY: all clean test test2 test3 test4 test5

all: doc_extractor.dylib ast_extractor

clean:
	rm -rv *dylib*

%.dylib: src/%.rs
	$(RUSTC) $< -o $@

ast_extractor: src/ast_extractor.rs src/visitor.rs
	$(RUSTC) $< -o $@

test: all
	$(RUSTDOC) --plugin-path . --plugins doc_extractor $(RUST_SRC_DIR)/src/libcollections/lib.rs


test2:
	$(RUSTDOC) --plugin-path . --plugins doc_extractor $(RUST_SRC_DIR)/src/libregex/lib.rs

test3:
	$(RUSTDOC) --plugin-path . --plugins doc_extractor $(RUST_SRC_DIR)/src/liblibc/lib.rs


test4:
	$(RUSTDOC) --plugin-path . --plugins doc_extractor $(RUST_SRC_DIR)/src/libgraphviz/lib.rs


test5:
	$(RUSTDOC) --plugin-path . --plugins doc_extractor ../rust-sdl2/src/sdl2/lib.rs
