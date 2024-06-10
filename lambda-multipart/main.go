package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/s3"
)

func main() {
	lambda.Start(handler)
}

func handler(ctx context.Context, request events.APIGatewayProxyRequest) (events.APIGatewayProxyResponse, error) {
	if request.HTTPMethod != http.MethodPost {
		return createResponse(http.StatusMethodNotAllowed, "Method not allowed", nil), fmt.Errorf("method not allowed")
	}

	bucket, region := LoadConfig()
	key := request.QueryStringParameters["key"]
	if key == "" {
		return createResponse(http.StatusBadRequest, "key parameter is missing", nil), fmt.Errorf("key parameter is mission")
	}

	sess := session.Must(session.NewSession())
	s3Svc := s3.New(sess, &aws.Config{
		Region: aws.String(region),
	})

	switch request.Path {
	case "/getpresignedurls":
		return handleGetPresignedUrls(s3Svc, bucket, key)
	case "/completeupload":
		return handleCompleteUpload(s3Svc, bucket, key, request.QueryStringParameters["uploadId"])
	case "/abortupload":
		return handleAbortUpload(s3Svc, bucket, key, request.QueryStringParameters["uploadId"])
	default:
		return createResponse(http.StatusNotFound, "Invalid endpoint", nil), fmt.Errorf("invalid endpoint %s", request.Path)
	}
}

func handleGetPresignedUrls(s3Svc *s3.S3, bucket string, key string) (events.APIGatewayProxyResponse, error) {
	const parts = 10
	requests := make([]string, 0, parts)

	createUploadResp, err := s3Svc.CreateMultipartUpload(&s3.CreateMultipartUploadInput{
		Bucket: aws.String(bucket),
		Key:    aws.String(key),
	})
	if err != nil {
		log.Printf("failed to create multipart upload: %v\n", err)
		return createResponse(500, "Failed to create multipart upload", nil), err
	}

	for i := 1; i <= parts; i++ {
		req, _ := s3Svc.UploadPartRequest(&s3.UploadPartInput{
			Bucket:     aws.String(bucket),
			Key:        aws.String(key),
			PartNumber: aws.Int64(int64(i)),
			UploadId:   createUploadResp.UploadId, // UploadId
		})

		url, err := req.Presign(15 * time.Hour)
		if err != nil {
			log.Printf("failed to create presign url: %v\n", err)
			return createResponse(500, "Failed to create presign url", nil), err
		}

		requests = append(requests, url)
	}

	body, err := json.Marshal(requests)
	if err != nil {
		log.Printf("failed to marshal presigned URLs: %v\n", err)
		return createResponse(500, "Failed to marshal presigned URLs", nil), err
	}

	return createResponse(200, string(body), nil), nil
}

func handleCompleteUpload(s3Svc *s3.S3, bucket, key, uploadID string) (events.APIGatewayProxyResponse, error) {
	if uploadID == "" {
		return createResponse(http.StatusBadRequest, "uploadId parameter is missing", nil), fmt.Errorf("uploadId parameter is missing")
	}

	completeResp, err := s3Svc.CompleteMultipartUpload(&s3.CompleteMultipartUploadInput{
		Bucket:   aws.String(bucket),
		Key:      aws.String(key),
		UploadId: aws.String(uploadID),
	})
	if err != nil {
		log.Printf("failed to complete multipart upload: %v\n", err)
		return createResponse(http.StatusInternalServerError, "Failed to complete multipart upload", nil), err
	}

	body, err := json.Marshal(completeResp)
	if err != nil {
		log.Printf("failed to marshal complete upload response: %v\n", err)
		return createResponse(http.StatusInternalServerError, "Failed to process request", nil), err
	}

	return createResponse(http.StatusOK, string(body), nil), nil
}

func handleAbortUpload(s3Svc *s3.S3, bucket, key, uploadID string) (events.APIGatewayProxyResponse, error) {
	if uploadID == "" {
		return createResponse(http.StatusBadRequest, "uploadId parameter is missing", nil), fmt.Errorf("uploadId parameter is missing")
	}

	_, err := s3Svc.AbortMultipartUpload(&s3.AbortMultipartUploadInput{
		Bucket:   aws.String(bucket),
		Key:      aws.String(key),
		UploadId: aws.String(uploadID),
	})
	if err != nil {
		log.Printf("failed to abort multipart upload: %v\n", err)
		return createResponse(http.StatusInternalServerError, "Failed to abort multipart upload", nil), err
	}

	return createResponse(http.StatusOK, "Multipart upload aborted successfully", nil), nil
}

func LoadConfig() (string, string) {
	bucket := os.Getenv("BUCKET_NAME")
	region := os.Getenv("AWS_REGION")

	if bucket == "" || region == "" {
		panic("BUCKET_NAME and AWS_REGION env var must be set")
	}

	return bucket, region
}

func createResponse(statusCode int, body string, headers map[string]string) events.APIGatewayProxyResponse {
	if headers == nil {
		headers = make(map[string]string)
	}
	headers["Content-Type"] = "application/json"

	return events.APIGatewayProxyResponse{
		StatusCode: statusCode,
		Body:       body,
		Headers:    headers,
	}
}
