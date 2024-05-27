import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { S3BucketStack } from "./s3bucket-stack";
import { ServiceStack } from "./service-stack";
import { ApiGatewayStack } from "./api_gateway-stack";

export class CmsLambdaStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const { bucket } = new S3BucketStack(this, "cmsImages");

    const {
      S3ResourcesLambda,
      // Other lambdas
    } = new ServiceStack(this, "cmsServices", {
      bucket: bucket.bucketName,
    });

    // Grant different access to only lambdas with s3 needed
    // Grant only read access to s3DownloadService
    bucket.grantRead(S3ResourcesLambda);

    new ApiGatewayStack(this, "cmsApiGateway", {
      S3ResourcesLambda,
    });
  }
}
