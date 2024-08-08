.PHONY: test
test:
	cargo test

.PHONY: sync-test-output
sync-test-output:
	cp testdata/output/* testdata/
