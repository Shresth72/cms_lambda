#/bin/bash

GOARCH=amd64 GOOS=linux go build -o bin/bootstrap main.go

rm -rf bin/lambdaPresigned.zip
zip lambdaPresigned.zip bootstrap

mv lambdaPresigned.zip bootstrap
