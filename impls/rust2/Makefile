STEP0_BIN = target/debug/step0_repl
STEP1_BIN = target/debug/step1_read_print

.PHONY: all test step0 step1 clean

all: $(STEP0_BIN) $(STEP1_BIN)

$(STEP0_BIN) $(STEP1_BIN):
	cargo build

step0: $(STEP0_BIN)
	cargo build
	STEP=step0_repl python3 ../../runtest.py ../../tests/step0_repl.mal ./run

step1: $(STEP1_BIN)
	cargo build
	STEP=step1_read_print python3 ../../runtest.py ../../tests/step1_read_print.mal ./run

clean:
	cargo clean

test: step0 step1 