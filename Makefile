YARN_FLAGS=
CARGO=cargo
CARGO_FLAGS=

ifeq ($(APP_ENVIRONMENT),prod)
	TARGET=target/release/wiki-md
	YARN_FLAGS+=--production
	CARGO_FLAGS+=--release
else
	TARGET=target/debug/wiki-md
endif

.DEFAULT_GOAL := build

build: $(TARGET) static/lib
.PHONY: build

$(TARGET):
	$(CARGO) build $(CARGO_FLAGS)

static/lib: package.json
	yarn install $(YARN_FLAGS)
