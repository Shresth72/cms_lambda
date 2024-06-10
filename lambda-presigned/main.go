package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/Shresth72/lambda-presigned/utils"
	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/s3"
)

func main() {
	lambda.Start(handler)
}

type PresignResponse struct {
	URL    string      `json:"url"`
	Header http.Header `json:"headers"`
}

func handler(ctx context.Context, request events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
	bucket, region := utils.LoadConfig()

	var url string
	var signedHeaders http.Header
  var err error

  // Bucket Key in the QueryParams
	key := request.QueryStringParameters["key"]
	if key == "" {
		return events.APIGatewayProxyResponse{
			StatusCode: 400,
			Body:       "key parameter is missing",
		}, fmt.Errorf("key parameter is missing")
	}

	sess := session.Must(session.NewSession())
	s3Svc := s3.New(sess, &aws.Config{
		Region: aws.String(region),
	})

	switch request.HTTPMethod {
	case "GET":
		sdkReq, _ := s3Svc.GetObjectRequest(&s3.GetObjectInput{
			Bucket: aws.String(bucket),
			Key:    aws.String(key),
		})
		url, signedHeaders, err = sdkReq.PresignRequest(15 * time.Minute)

	case "PUT":
		sdkReq, _ := s3Svc.PutObjectRequest(&s3.PutObjectInput{
			Bucket: aws.String(bucket),
			Key:    aws.String(key),
		})
		url, signedHeaders, err = sdkReq.PresignRequest(15 * time.Minute)

  case "DELETE":
    sdkReq, _ := s3Svc.DeleteObjectRequest(&s3.DeleteObjectInput{
      Bucket: aws.String(bucket),
      Key: aws.String(key),
    })
		url, signedHeaders, err = sdkReq.PresignRequest(15 * time.Minute)

	default:
		log.Print("invalid method provided\n", request.HTTPMethod, err)
		err = fmt.Errorf("invalid request %v", err)
	}

	if err != nil {
		return events.APIGatewayProxyResponse{
			StatusCode: 500,
			Body:       "",
		}, err
	}

	return utils.CreateResponse(200, PresignResponse{
		URL:    url,
		Header: signedHeaders,
	}), nil
}
