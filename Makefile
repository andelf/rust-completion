.PHONY: all test test2

all: doc_extractor.dylib



%.dylib: src/%.rs
	rustc -g $< -o $@


test: all
	rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs


test2:
	rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libregex/lib.rs
