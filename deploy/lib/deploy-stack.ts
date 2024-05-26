import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { S3BucketStack } from "./s3bucket-stack";
import { ServiceStack } from "./service-stack";
import { ApiGatewayStack } from "./api-gateway-stack";

export class DeployStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const { bucket } = new S3BucketStack(this, "cmsImages");

    const {
      s3DownloadService,
      // Other lambdas
    } = new ServiceStack(this, "cmsServices", {
      bucket: bucket.bucketName,
    });

    // Grant different access to only lambdas with s3 needed
    // Grant only read access to s3DownloadService
    bucket.grantRead(s3DownloadService);

    new ApiGatewayStack(this, "cmsApiGateway", {
      s3DownloadService,
    });
  }
}
