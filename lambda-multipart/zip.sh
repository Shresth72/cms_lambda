#/bin/bash

GOARCH=amd64 GOOS=linux go build -o bin/bootstrap main.go

rm -rf bin/lambdaMultipart.zip
zip lambdaMultipart.zip bootstrap

mv lambdaMultipart.zip bootstrap
