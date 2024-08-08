.PHONY: test
test:
	cargo test

.PHONY: sync-test-output
sync-test-output:
	cp testdata/output/* testdata/

.PHONY: run
run:
	cargo run -- jobs --work-dir ./testdata/manual_cli/base/repo3_work/ --config-path ./testdata/manual_cli/config.yaml
