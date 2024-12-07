STEP0_BIN = target/debug/step0_repl

.PHONY: all test step0 clean

all: $(STEP0_BIN)

$(STEP0_BIN):
	cargo build

step0: $(STEP0_BIN)
	cargo build
	python3 ../../runtest.py ./run

clean:
	cargo clean

test: step0 