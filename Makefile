
SRCS = lib.rs lazy.rs shrink.rs arbitrary.rs

qc: $(SRCS)
	rustc --test -o qc $<

libqc: $(SRCS)
	rustc $<

test: qc
	./qc

.PHONY: test
