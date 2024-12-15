build:
	go build -ldflags="-s -w" -o bin/bot bot/main.go

clean:
	rm -rf ./bin ./vendor
