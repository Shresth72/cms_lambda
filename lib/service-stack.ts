import { Duration } from "aws-cdk-lib";
import {
  Function,
  Runtime,
  Code,
  FunctionProps,
  Architecture,
} from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";
import { join } from "path";

// TODO: The s3 downloader is just temporary, change the names of all the stuff
// And add more lambdas later
interface ServiceProps {
  bucket: string;
}

export class ServiceStack extends Construct {
  public readonly S3ResourcesLambda: Function;
  public readonly MultiPartLambda: Function;

  constructor(scope: Construct, id: string, props: ServiceProps) {
    super(scope, id);

    this.S3ResourcesLambda = new Function(this, "S3ResourcesLambda", {
      description: "S3 Resource Manager Lambda for smaller files",
      code: Code.fromAsset(
        "lambda-resources/target/X86_64-unknown-linux-musl/release/lambda",
      ),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.X86_64,
      timeout: Duration.seconds(10),
      handler: "bootstrap",
      environment: {
        RUST_BACKTRACE: "1",
        BUCKET_NAME: props.bucket,
      },
    });

    this.MultiPartLambda = new Function(this, "MultiPartLambda", {
      description: "S3 Resource Manager Lambda for larger files",
      code: Code.fromAsset(
        "lambda-multipart/target/X86_64-unknown-linux-musl/release/lambda",
      ),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.X86_64,
      timeout: Duration.seconds(10),
      handler: "bootstrap",
      environment: {
        RUST_BACKTRACE: "1",
        BUCKET_NAME: props.bucket,
      },
    });
  }
}
