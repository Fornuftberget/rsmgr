.PHONY: test
test:
	@scripts/test-env.sh > /dev/null
	@echo "#git_clone repo1:repo1" > test/server.cfg
	@echo "#git_clone repo2:repo2" >> test/server.cfg
	@echo "#git_clone repo3:repo3" >> test/server.cfg
	cargo run -- --discard --base_path ./test/local --base_url "file://$(PWD)/test/origin/" --config test/server.cfg
	@scripts/test-commits.sh > /dev/null
	cargo run -- --verbose --discard --base_path ./test/local --base_url "file://$(PWD)/test/origin/" --config test/server.cfg

clean:
	rm test
	rm -rf targets
