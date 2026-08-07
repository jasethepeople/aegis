.PHONY: all build server worker test clean

all: build

build: server worker

server:
	go build -o bin/aegis-server ./cmd/server

worker:
	go build -o bin/aegis-worker ./cmd/server

test:
	go test -v ./...

clean:
	rm -rf bin/

fmt:
	go fmt ./...

deps:
	go mod download
	go mod tidy
