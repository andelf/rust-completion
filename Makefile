.PHONY: all test

all: doc_extractor.dylib



%.dylib: src/%.rs
	rustc $< -o $@


test: all
	rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs
