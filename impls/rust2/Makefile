STEP0_BIN = target/debug/step0_repl
STEP1_BIN = target/debug/step1_read_print
STEP2_BIN = target/debug/step2_eval
STEP3_BIN = target/debug/step3_env
STEP4_BIN = target/debug/step4_if_fn_do

.PHONY: all test step0 step1 step2 step3 step4 clean

all: $(STEP0_BIN) $(STEP1_BIN) $(STEP2_BIN) $(STEP3_BIN) $(STEP4_BIN)

$(STEP0_BIN) $(STEP1_BIN) $(STEP2_BIN) $(STEP3_BIN) $(STEP4_BIN):
	cargo build

step0: $(STEP0_BIN)
	cargo build
	STEP=step0_repl python3 ../../runtest.py ../../tests/step0_repl.mal ./run

step1: $(STEP1_BIN)
	cargo build
	STEP=step1_read_print python3 ../../runtest.py ../../tests/step1_read_print.mal ./run

step2: $(STEP2_BIN)
	cargo build
	STEP=step2_eval python3 ../../runtest.py ../../tests/step2_eval.mal ./run

step3: $(STEP3_BIN)
	cargo build
	STEP=step3_env python3 ../../runtest.py ../../tests/step3_env.mal ./run

step4: $(STEP4_BIN)
	cargo build
	STEP=step4_if_fn_do python3 ../../runtest.py ../../tests/step4_if_fn_do.mal ./run

clean:
	cargo clean

test: step0 step1 step2 step3 step4 