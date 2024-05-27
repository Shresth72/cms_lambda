import { aws_apigateway } from "aws-cdk-lib";
import {
  LambdaIntegration,
  LambdaRestApi,
  RestApi,
} from "aws-cdk-lib/aws-apigateway";
import { IFunction } from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";

interface ApiGatewayStackProps {
  S3ResourcesLambda: IFunction;
}

interface ResourceType {
  name: string;
  methods: string[];
  child?: ResourceType;
}

export class ApiGatewayStack extends Construct {
  constructor(scope: Construct, id: string, props: ApiGatewayStackProps) {
    super(scope, id);
  }

  addResource(
    serviceName: string,
    { S3ResourcesLambda }: ApiGatewayStackProps,
  ) {
    const apigw = new aws_apigateway.RestApi(this, `${serviceName}-ApiGtw`);

    this.createEndPoints(S3ResourcesLambda, apigw, {
      name: "resources",
      methods: ["GET", "PUT"],
    });
  }

  createEndPoints(
    handler: IFunction,
    resource: RestApi,
    { name, methods, child }: ResourceType,
  ) {
    const lambdaFunction = new LambdaIntegration(handler);
    const rootResource = resource.root.addResource(name);
    methods.map((item) => {
      rootResource.addMethod(item, lambdaFunction);
    });

    if (child) {
      const childResource = rootResource.addResource(child.name);
      child.methods.map((item) => {
        childResource.addMethod(item, lambdaFunction);
      });
    }
  }
}
