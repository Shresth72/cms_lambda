import { Duration } from "aws-cdk-lib";
import { Function, Runtime, Code, FunctionProps } from "aws-cdk-lib/aws-lambda";
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
      code: Code.fromAsset("../../s3_download/target/lambda/s3_download"),
      runtime: Runtime.PROVIDED_AL2,
      timeout: Duration.seconds(10),
      handler: "nil",
      environment: {
        BUCKET_NAME: props.bucket,
      },
    });
  }
}
