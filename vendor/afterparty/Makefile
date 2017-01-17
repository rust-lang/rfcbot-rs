build:
	@cargo build
.PHONY: build

generate: export FETCH_PAYLOAD_DATA=true
generate:
	@cargo build
.PHONY: generate

clean:
	@cargo clean
	$(shell rm data/*.json)
.PHONY: clean

test:
	@cargo test
.PHONY: test
