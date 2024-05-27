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
  public readonly s3DownloadService: Function;

  constructor(scope: Construct, id: string, props: ServiceProps) {
    super(scope, id);

    this.s3DownloadService = new Function(this, "s3DownloadLambda", {
      description: "S3 Download Rust function on lambda using custom runtime",
      code: Code.fromAsset(""),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.X86_64,
      timeout: Duration.seconds(10),
      handler: "not.required",
      environment: {
        RUST_BACKTRACE: "1",
        BUCKET_NAME: props.bucket,
      },
    });
  }
}
