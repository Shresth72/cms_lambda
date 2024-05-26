import { RemovalPolicy } from "aws-cdk-lib";
import {
  BlockPublicAccess,
  Bucket,
  BucketEncryption,
} from "aws-cdk-lib/aws-s3";
import { Construct } from "constructs";
import { v4 as uuidv4 } from "uuid";

export class S3BucketStack extends Construct {
  public readonly bucket: Bucket;

  constructor(scope: Construct, name: string) {
    super(scope, name);

    this.bucket = new Bucket(scope, `${name}-${uuidv4()}-bucket`, {
      blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
      encryption: BucketEncryption.S3_MANAGED,
      enforceSSL: true,
      versioned: false,
      removalPolicy: RemovalPolicy.DESTROY,
    });
  }
}
