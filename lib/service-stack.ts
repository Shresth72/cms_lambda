import { Duration } from "aws-cdk-lib";
import {
  Function,
  Runtime,
  Code,
  FunctionProps,
  Architecture,
  Tracing,
} from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";
import { join } from "path";

interface ServiceProps {
  bucket: string;
}

export class ServiceStack extends Construct {
  public readonly S3PresignedLambda: Function;
  public readonly MultiPartLambda: Function;

  constructor(scope: Construct, id: string, props: ServiceProps) {
    super(scope, id);

    this.S3PresignedLambda = new Function(this, "S3PresignedLambda", {
      functionName: "s3-presigned",
      description: "S3 Resource Manager Lambda for smaller files",
      code: Code.fromAsset(
        "/home/shrestha/rust/cms_lambda/lambda-presigned/bootstrap",
      ),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.X86_64,
      timeout: Duration.seconds(10),
      handler: "bootstrap",
      environment: {
        BUCKET_NAME: props.bucket,
      },
      tracing: Tracing.ACTIVE,
    });

    this.MultiPartLambda = new Function(this, "MultiPartLambda", {
      functionName: "s3-multipart",
      description: "S3 Resource Manager Lambda for larger files",
      code: Code.fromAsset(
        "/home/shrestha/rust/cms_lambda/lambda-multipart/bootstrap",
      ),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.X86_64,
      timeout: Duration.seconds(10),
      handler: "bootstrap",
      environment: {
        BUCKET_NAME: props.bucket,
      },
    });
  }
}
