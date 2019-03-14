BOWER_FLAGS=
CARGO=cargo
CARGO_FLAGS=

ifeq ($(APP_ENVIRONMENT),prod)
	TARGET=target/release/wiki-md
	BOWER_FLAGS+=--production
	CARGO_FLAGS+=--release
else
	TARGET=target/debug/wiki-md
endif

.DEFAULT_GOAL := build

build: $(TARGET) static/lib

$(TARGET):
	$(CARGO) build $(CARGO_FLAGS)

static/lib: bower.json
	bower install $(BOWER_FLAGS)
