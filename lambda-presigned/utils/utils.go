package utils

import (
	"encoding/json"
	"log"
	"os"

	"github.com/aws/aws-lambda-go/events"
)

func CreateResponse(statusCode int, body interface{}) events.APIGatewayProxyResponse {
	jsonBody, err := json.Marshal(body)
	if err != nil {
		log.Printf("failed to marshal response body %v", err)
		return events.APIGatewayProxyResponse{
			StatusCode: 500,
			Body:       `{"error": "Internal Server Error"}`,
		}
	}

	return events.APIGatewayProxyResponse{
		StatusCode: statusCode,
		Body:       string(jsonBody),
	}
}

func LoadConfig() (string, string) {
	bucket := os.Getenv("BUCKET_NAME")
	region := os.Getenv("AWS_REGION")

	if bucket == "" || region == "" {
		panic("BUCKET_NAME and AWS_REGION env var must be set")
	}

	return bucket, region

}
